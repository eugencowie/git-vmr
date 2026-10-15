use super::{Declaration, Manifest, Repo};
use crate::git::git_style_path;
use anyhow::{Context, Result, bail};
use std::path::{Component, Path, PathBuf};
use std::{fs, iter};

/// A VMR root's membership: the root repo and the cloned child repos are
/// member repos; missing child repos (declared, not on disk) and undeclared
/// repos (first-level Git repositories the manifest does not declare) are
/// not. The only place that decides which Git repositories sit at a VMR
/// root, and which of them owns a path within it.
#[derive(Debug)]
pub struct Membership
{
    /// The root repo, then the cloned child repos sorted by path.
    repos: Vec<Repo>,
    missing: Vec<Declaration>,
    undeclared: Vec<String>
}

/// What owns a path within the VMR root.
#[derive(Debug, PartialEq)]
pub enum Classification<'a>
{
    /// Inside a member repo — a cloned child repo, otherwise the root repo —
    /// with the path relative to it.
    Member(&'a Repo, PathBuf),
    /// A cloned child repo's root directory.
    ChildRoot(&'a Repo),
    /// Inside a first-level Git repository the manifest does not declare,
    /// with the path relative to it.
    Undeclared
    {
        name: &'a str, path: PathBuf
    }
}

impl Membership
{
    /// Reads `root`'s first level once and sorts the Git repositories
    /// there against `manifest`.
    pub fn read(root: &Path, manifest: &Manifest) -> Result<Membership>
    {
        // List the first-level Git repositories
        let on_disk = fs::read_dir(root)
            .with_context(|| {
                format!("fatal: failed to read '{}'", root.display())
            })?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .filter(|entry| entry.path().join(".git").exists())
            // Names that are not valid UTF-8 cannot be declared
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name != ".git");

        // Sort them against the manifest
        Ok(Membership::new(root, manifest, on_disk))
    }

    /// Sorts `on_disk`, the names of `root`'s first-level Git
    /// repositories, against `manifest`.
    pub fn new(
        root: &Path,
        manifest: &Manifest,
        on_disk: impl IntoIterator<Item = String>
    ) -> Membership
    {
        // Split the repos on disk into declared and undeclared
        let declarations = manifest.declarations();
        let (declared, mut undeclared): (Vec<String>, Vec<String>) =
            on_disk.into_iter().partition(|name| {
                declarations
                    .iter()
                    .any(|declaration| declares(declaration, name))
            });
        undeclared.sort();

        // Split the declarations into cloned and missing
        let (cloned, missing): (Vec<&Declaration>, Vec<&Declaration>) =
            declarations.iter().partition(|declaration| {
                declared.iter().any(|name| declares(declaration, name))
            });

        // Hold the root repo, then the cloned child repos
        Membership {
            repos: iter::once(Repo::root(root))
                .chain(
                    cloned
                        .into_iter()
                        .map(|declaration| Repo::child(root, declaration))
                )
                .collect(),
            missing: missing.into_iter().cloned().collect(),
            undeclared
        }
    }

    /// The root repo, which owns every path outside the child repos.
    pub fn root(&self) -> &Repo
    {
        &self.repos[0]
    }

    /// The member repos a VMR operation runs across: the root repo, then
    /// the cloned child repos sorted by path.
    pub fn members(&self) -> &[Repo]
    {
        &self.repos
    }

    /// The declared child repos that are cloned, sorted by path.
    pub fn children(&self) -> &[Repo]
    {
        &self.repos[1..]
    }

    /// The declared child repos that are not cloned, sorted by path.
    pub fn missing(&self) -> &[Declaration]
    {
        &self.missing
    }

    /// The names of first-level Git repositories the manifest does not
    /// declare, sorted.
    pub fn undeclared(&self) -> &[String]
    {
        &self.undeclared
    }

    /// Classifies `path`, relative to the VMR root, by what owns it: a
    /// member repo, or an undeclared repo, with the path relative to it, or
    /// a child root itself. Fails for a path inside a missing child repo or
    /// a Git repository nested below the first level.
    pub fn classify(&self, path: &Path) -> Result<Classification<'_>>
    {
        // Take the first path component as the first-level entry; the VMR
        // root itself belongs to the root repo
        let mut components = path.components();
        let Some(Component::Normal(entry)) = components.next()
        else
        {
            return Ok(Classification::Member(self.root(), PathBuf::from(".")));
        };
        let rest = components.as_path();

        // Names that are not valid UTF-8 belong to no repo
        let Some(entry) = entry.to_str()
        else
        {
            return self.classify_root_path(path);
        };

        // Look for a cloned repo, then a missing one, then an undeclared one
        if let Some(repo) =
            self.children().iter().find(|repo| repo.path == entry)
        {
            return Ok(
                if rest.as_os_str().is_empty()
                {
                    Classification::ChildRoot(repo)
                }
                else
                {
                    Classification::Member(repo, rest.to_path_buf())
                }
            );
        }
        if let Some(declaration) =
            self.missing.iter().find(|declaration| declares(declaration, entry))
        {
            bail!(
                "error: '{}' is not cloned; run 'git vmr clone' to clone it",
                declaration.path
            );
        }
        if let Some(name) =
            self.undeclared.iter().find(|undeclared| *undeclared == entry)
        {
            return Ok(Classification::Undeclared {
                name,
                path: dot_if_empty(rest)
            });
        }

        // Anything else belongs to the root repo
        self.classify_root_path(path)
    }

    /// Classifies `path` as the root repo's, unless it lies inside a Git
    /// repository nested below the first level, which the root repo must
    /// not silently embed.
    fn classify_root_path(&self, path: &Path) -> Result<Classification<'_>>
    {
        // Look for a nested repo, outermost first, skipping the VMR root
        // and the first level, whose repos were listed when reading
        let ancestors = path.ancestors().collect::<Vec<_>>();
        if let Some(nested) = ancestors.iter().rev().skip(2).find(|ancestor| {
            self.root().absolute_path.join(ancestor).join(".git").exists()
        })
        {
            bail!(
                "error: '{}' is inside nested Git repository '{}', which is \
                 not a member repo",
                git_style_path(path),
                git_style_path(nested)
            );
        }

        // Otherwise the path is the root repo's
        Ok(Classification::Member(self.root(), path.to_path_buf()))
    }
}

/// Whether `declaration` declares the first-level entry `name`.
fn declares(declaration: &Declaration, name: &str) -> bool
{
    declaration.path.as_str() == name
}

/// `path`, or `.` when it is empty.
fn dot_if_empty(path: &Path) -> PathBuf
{
    if path.as_os_str().is_empty()
    {
        PathBuf::from(".")
    }
    else
    {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{manifest, repo};

    fn repo_paths(repos: &[Repo]) -> Vec<&str>
    {
        repos.iter().map(|repo| repo.path.as_str()).collect()
    }

    fn paths(declarations: &[Declaration]) -> Vec<&str>
    {
        declarations
            .iter()
            .map(|declaration| declaration.path.as_str())
            .collect()
    }

    fn on_disk(names: &[&str]) -> Vec<String>
    {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn sorts_repos_into_cloned_missing_and_undeclared()
    {
        // Arrange
        let root = Path::new("/vmr");
        let manifest = manifest(&["api", "backend", "frontend"]);

        // Act
        let membership = Membership::new(
            root,
            &manifest,
            on_disk(&["scratch", "frontend", "backend"])
        );

        // Assert
        assert_eq!(membership.children(), [
            repo("backend", root.join("backend")),
            repo("frontend", root.join("frontend"))
        ]);
        assert_eq!(paths(membership.missing()), ["api"]);
        assert_eq!(membership.undeclared(), ["scratch"]);
    }

    #[test]
    fn makes_the_root_repo_and_the_cloned_children_members()
    {
        // Arrange
        let manifest = manifest(&["api", "backend", "frontend"]);

        // Act
        let membership = Membership::new(
            Path::new("/vmr"),
            &manifest,
            on_disk(&["scratch", "frontend", "backend"])
        );

        // Assert
        assert_eq!(repo_paths(membership.members()), [
            ".", "backend", "frontend"
        ]);
    }

    #[test]
    fn gives_cloned_repos_their_manifest_path_sorted_by_path()
    {
        // Arrange: names sort the other way round from paths
        let manifest = Manifest::parse(
            "[repo.zeta]\npath = \"a\"\nurl = \"u\"\n\n\
             [repo.alpha]\npath = \"b\"\nurl = \"u\"\n"
        )
        .unwrap();

        // Act
        let membership =
            Membership::new(Path::new("/vmr"), &manifest, on_disk(&["b", "a"]));

        // Assert
        assert_eq!(repo_paths(membership.children()), ["a", "b"]);
    }

    fn membership() -> Membership
    {
        Membership::new(
            Path::new("/vmr"),
            &manifest(&["api", "backend", "frontend"]),
            on_disk(&["backend", "frontend", "scratch"])
        )
    }

    #[test]
    fn classifies_a_path_inside_a_child_repo_relative_to_it()
    {
        // Arrange
        let membership = membership();

        // Act
        let class = membership.classify(Path::new("backend/src/main.rs"));

        // Assert
        assert_eq!(
            class.unwrap(),
            Classification::Member(
                &repo("backend", "/vmr/backend"),
                PathBuf::from("src/main.rs")
            )
        );
    }

    #[test]
    fn classifies_a_child_root()
    {
        // Arrange
        let membership = membership();

        // Act
        let class = membership.classify(Path::new("backend"));

        // Assert
        assert_eq!(
            class.unwrap(),
            Classification::ChildRoot(&repo("backend", "/vmr/backend"))
        );
    }

    #[test]
    fn classifies_paths_outside_every_child_repo_as_the_root_repo()
    {
        // Arrange: docs is not a repo, README.md is a root file
        let membership = membership();

        for (path, root_path) in [
            ("", "."),
            ("README.md", "README.md"),
            ("docs/guide.md", "docs/guide.md"),
            (".gitvmr", ".gitvmr")
        ]
        {
            // Act
            let class = membership.classify(Path::new(path));

            // Assert
            assert_eq!(
                class.unwrap(),
                Classification::Member(
                    &Repo::root(Path::new("/vmr")),
                    PathBuf::from(root_path)
                )
            );
        }
    }

    #[test]
    fn classifies_a_path_inside_an_undeclared_repo_relative_to_it()
    {
        // Arrange
        let membership = membership();

        for (path, repo_path) in
            [("scratch/notes.md", "notes.md"), ("scratch", ".")]
        {
            // Act
            let class = membership.classify(Path::new(path));

            // Assert
            assert_eq!(class.unwrap(), Classification::Undeclared {
                name: "scratch",
                path: PathBuf::from(repo_path)
            });
        }
    }

    #[test]
    fn classifying_a_path_inside_a_missing_child_repo_fails()
    {
        // Arrange
        let membership = membership();

        // Act
        let err = membership.classify(Path::new("api/src")).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: 'api' is not cloned; run 'git vmr clone' to clone it"
        );
    }

    #[test]
    fn classifying_a_root_path_inside_a_nested_repo_fails()
    {
        // Arrange: vendor/lib is a Git repository below the first level
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/lib/.git")).unwrap();
        let membership =
            Membership::new(tmp.path(), &Manifest::default(), on_disk(&[]));

        for path in ["vendor/lib", "vendor/lib/src/a.rs"]
        {
            // Act
            let err = membership.classify(Path::new(path)).unwrap_err();

            // Assert
            assert_eq!(
                err.to_string(),
                format!(
                    "error: '{path}' is inside nested Git repository \
                     'vendor/lib', which is not a member repo"
                )
            );
        }
    }

    #[test]
    fn classifies_a_root_path_beside_a_nested_repo_as_the_root_repo()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/lib/.git")).unwrap();
        let membership =
            Membership::new(tmp.path(), &Manifest::default(), on_disk(&[]));

        // Act
        let class = membership.classify(Path::new("vendor/README.md"));

        // Assert
        assert_eq!(
            class.unwrap(),
            Classification::Member(
                &Repo::root(tmp.path()),
                PathBuf::from("vendor/README.md")
            )
        );
    }

    #[test]
    fn reads_only_real_directories_holding_git_as_repos()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
        fs::create_dir_all(tmp.path().join("linked")).unwrap();
        fs::write(tmp.path().join("linked/.git"), "gitdir: elsewhere\n")
            .unwrap();
        fs::create_dir_all(tmp.path().join("docs")).unwrap();
        fs::write(tmp.path().join("README.md"), "").unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            tmp.path().join("backend"),
            tmp.path().join("alias")
        )
        .unwrap();

        // Act
        let membership =
            Membership::read(tmp.path(), &Manifest::default()).unwrap();

        // Assert
        assert_eq!(membership.undeclared(), ["backend", "linked"]);
    }

    #[test]
    fn reading_a_missing_root_fails()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("gone");

        // Act
        let err = Membership::read(&root, &Manifest::default()).unwrap_err();

        // Assert
        assert!(format!("{err:#}").starts_with("fatal: failed to read"));
    }
}
