use super::{Declaration, Manifest, Repo};
use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// A VMR root's membership as far as child repos go: the cloned child
/// repos are member repos; missing child repos (declared, not on disk) and
/// undeclared repos (first-level Git repositories the manifest does not
/// declare) are not. The only place that decides which Git repositories
/// sit at a VMR root.
#[derive(Debug)]
pub struct Membership
{
    cloned: Vec<Repo>,
    missing: Vec<Declaration>,
    undeclared: Vec<String>
}

/// The repo a first-level entry of the VMR root belongs to.
#[derive(Debug, Eq, PartialEq)]
pub enum Owner<'a>
{
    Cloned(&'a Repo),
    Missing(&'a Declaration),
    Undeclared
}

impl Membership
{
    /// Reads `root`'s first level once and sorts the Git repositories
    /// there against `manifest`.
    pub fn read(root: &Path, manifest: &Manifest) -> Result<Membership>
    {
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
        let declarations = manifest.declarations();
        let (declared, mut undeclared): (Vec<String>, Vec<String>) =
            on_disk.into_iter().partition(|name| {
                declarations
                    .iter()
                    .any(|declaration| declares(declaration, name))
            });
        undeclared.sort();

        let (cloned, missing): (Vec<&Declaration>, Vec<&Declaration>) =
            declarations.iter().partition(|declaration| {
                declared.iter().any(|name| declares(declaration, name))
            });

        Membership {
            cloned: cloned
                .into_iter()
                .map(|declaration| Repo {
                    name: declaration.path.to_string(),
                    path: root.join(declaration.path.as_str())
                })
                .collect(),
            missing: missing.into_iter().cloned().collect(),
            undeclared
        }
    }

    /// The declared child repos that are cloned, sorted by path.
    pub fn cloned(&self) -> &[Repo]
    {
        &self.cloned
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

    /// The repo that the first-level entry `name` belongs to, if any.
    pub fn owner(&self, name: &OsStr) -> Option<Owner<'_>>
    {
        let name = name.to_str()?;
        if let Some(repo) = self.cloned.iter().find(|repo| repo.name == name)
        {
            return Some(Owner::Cloned(repo));
        }
        if let Some(declaration) =
            self.missing.iter().find(|declaration| declares(declaration, name))
        {
            return Some(Owner::Missing(declaration));
        }
        self.undeclared
            .iter()
            .any(|undeclared| undeclared == name)
            .then_some(Owner::Undeclared)
    }
}

/// Whether `declaration` declares the first-level entry `name`.
fn declares(declaration: &Declaration, name: &str) -> bool
{
    declaration.path.as_str() == name
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::manifest;

    fn names(repos: &[Repo]) -> Vec<&str>
    {
        repos.iter().map(|repo| repo.name.as_str()).collect()
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
        assert_eq!(membership.cloned(), [
            Repo { name: "backend".to_owned(), path: root.join("backend") },
            Repo { name: "frontend".to_owned(), path: root.join("frontend") }
        ]);
        assert_eq!(paths(membership.missing()), ["api"]);
        assert_eq!(membership.undeclared(), ["scratch"]);
    }

    #[test]
    fn labels_cloned_repos_by_manifest_path_sorted_by_path()
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
        assert_eq!(names(membership.cloned()), ["a", "b"]);
    }

    #[test]
    fn finds_the_owner_of_a_first_level_entry()
    {
        // Arrange
        let manifest = manifest(&["api", "backend"]);
        let membership = Membership::new(
            Path::new("/vmr"),
            &manifest,
            on_disk(&["backend", "scratch"])
        );

        // Act
        let owners = ["backend", "api", "scratch", "docs"]
            .map(|name| membership.owner(OsStr::new(name)));

        // Assert
        assert_eq!(owners, [
            Some(Owner::Cloned(&membership.cloned()[0])),
            Some(Owner::Missing(&membership.missing()[0])),
            Some(Owner::Undeclared),
            None
        ]);
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
