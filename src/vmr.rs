mod repo;

use crate::config::Config;
use crate::store::FileStore;
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
pub use repo::Repo;
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

/// Whether `Vmr::init` created a new VMR root or found an existing one.
#[derive(Debug, Eq, PartialEq)]
pub enum InitOutcome
{
    Created,
    Reinitialized
}

#[derive(Debug)]
pub struct Vmr
{
    pub path: PathBuf
}

impl Vmr
{
    pub fn new(path: &Path) -> Vmr
    {
        Vmr { path: path.clean() }
    }

    pub fn find(working_dir: &Path) -> Result<Vmr>
    {
        // Search working directory and ancestors
        for dir in working_dir.ancestors()
        {
            if Vmr::is_root(dir)
            {
                return Ok(Vmr::new(dir));
            }
        }

        // Report missing VMR root
        bail!(
            "fatal: not a virtual monorepo (or any of the parent directories): .gitvmr"
        )
    }

    /// Whether a directory is a VMR root, of either kind: a main root
    /// (marker directory) or a worktree root (marker file).
    pub fn is_root(dir: &Path) -> bool
    {
        dir.join(".gitvmr").exists()
    }

    /// Initializes a main VMR root: a `.gitvmr` marker directory holding
    /// the default VMR config.
    pub fn init(target_dir: &Path) -> Result<InitOutcome>
    {
        let vmr_dir = target_dir.join(".gitvmr");
        let config_path = vmr_dir.join("config");

        if config_path.is_file()
        {
            return Ok(InitOutcome::Reinitialized);
        }

        fs::create_dir_all(&vmr_dir)
            .context("fatal: failed to create .gitvmr directory")?;

        FileStore::new(config_path, Config::default())
            .write()
            .context("fatal: failed to write .gitvmr/config")?;

        Ok(InitOutcome::Created)
    }

    /// Materializes a worktree root: the target directory with a `.gitvmr`
    /// marker file.
    pub fn create_worktree_root(target: &Path) -> Result<()>
    {
        fs::create_dir_all(target).with_context(|| {
            format!(
                "fatal: failed to create worktree target '{}'",
                target.display()
            )
        })?;

        fs::write(target.join(".gitvmr"), "").with_context(|| {
            format!(
                "fatal: failed to create VMR marker in '{}'",
                target.display()
            )
        })
    }

    /// Dissolves a worktree root: removes the marker of either kind, then
    /// the directory itself if it is empty.
    pub fn remove_worktree_root(target: &Path) -> Result<()>
    {
        let marker = target.join(".gitvmr");
        if marker.exists()
        {
            let removal = if marker.is_dir()
            {
                fs::remove_dir(&marker)
            }
            else
            {
                fs::remove_file(&marker)
            };
            removal.with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }

        match fs::remove_dir(target)
        {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) if error.kind() == ErrorKind::DirectoryNotEmpty =>
                Ok(()),
            Err(error) => Err(error).with_context(|| {
                format!(
                    "fatal: failed to remove empty worktree directory '{}'",
                    target.display()
                )
            })
        }
    }

    pub fn repos(&self) -> Result<Vec<Repo>>
    {
        let mut repos = Vec::new();

        // Scan immediate child directories
        for repo_path in fs::read_dir(&self.path)
            .with_context(|| {
                format!(
                    "fatal: failed to read VMR root '{}'",
                    self.path.display()
                )
            })?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
            })
            .filter(|entry| entry.file_name() != ".gitvmr")
            .map(|entry| entry.path())
            .collect::<Vec<_>>()
        {
            // Add children that are Git repositories
            if let Some(repo) = Repo::find(repo_path)
            {
                repos.push(repo);
            }
        }

        // Keep repository order deterministic
        repos.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(repos)
    }

    pub fn route_paths(
        &self,
        repos: &[Repo],
        working_dir: &Path,
        paths: &[PathBuf]
    ) -> Result<BTreeMap<Repo, Vec<PathBuf>>>
    {
        let mut grouped: BTreeMap<Repo, Vec<PathBuf>> = BTreeMap::new();

        // Resolve and route every path before mutating any repository
        for path in paths
        {
            let normalized = resolve_target(working_dir, path);
            let routed =
                self.route_path(repos, &normalized).with_context(|| {
                    format!("error: failed to route '{}'", path.display())
                })?;

            for (repo_path, repo_relative_path) in routed
            {
                grouped.entry(repo_path).or_default().push(repo_relative_path);
            }
        }

        Ok(grouped)
    }

    pub fn route_single_path(
        &self,
        repos: &[Repo],
        working_dir: &Path,
        path: &Path
    ) -> Result<(Repo, PathBuf)>
    {
        // Resolve one operand without allowing aggregate VMR root expansion
        let normalized = resolve_target(working_dir, path);

        if normalized == self.path
        {
            bail!(
                "error: '{}' resolves to the VMR root aggregate",
                path.display()
            );
        }

        // Reuse normal child repository ownership checks
        self.route_path(repos, &normalized)
            .with_context(|| {
                format!("error: failed to route '{}'", path.display())
            })?
            .into_iter()
            .next()
            .context("error: path did not route to a child repository")
    }

    fn route_path(
        &self,
        repos: &[Repo],
        path: &Path
    ) -> Result<Vec<(Repo, PathBuf)>>
    {
        // Expand the aggregate VMR root view across child repositories
        if path == self.path
        {
            return Ok(repos
                .iter()
                .map(|repo| (repo.clone(), PathBuf::from(".")))
                .collect());
        }

        // Ensure path stays inside the VMR root
        let relative = path.strip_prefix(&self.path).with_context(|| {
            format!(
                "error: '{}' is outside virtual monorepo '{}'",
                path.display(),
                self.path.display()
            )
        })?;

        // Use first path component as owning child repository
        let mut components = relative.components();
        let repo_name = match components.next()
        {
            Some(Component::Normal(name)) => name,
            _ =>
            {
                bail!(
                    "error: '{}' is not owned by a child repository",
                    path.display()
                );
            }
        };

        // Reject VMR metadata paths
        if repo_name == ".gitvmr"
        {
            bail!(
                "error: cannot route virtual monorepo metadata path '{}'",
                path.display()
            );
        }

        // Require explicit paths to be owned by snapshotted child repositories
        let repo_path = self.path.join(repo_name);
        let repo = repos
            .iter()
            .find(|repo| repo.path == repo_path)
            .cloned()
            .with_context(|| {
                format!(
                    "error: '{}' is not owned by a child Git repository",
                    path.display()
                )
            })?;
        // Build path relative to owning repository
        let mut repo_relative_path = PathBuf::new();
        for component in components
        {
            repo_relative_path.push(component.as_os_str());
        }
        if repo_relative_path.as_os_str().is_empty()
        {
            repo_relative_path.push(".");
        }

        Ok(vec![(repo, repo_relative_path)])
    }
}

/// Resolves a user-supplied path against the effective working directory
/// and normalizes it lexically. The only way user paths become filesystem
/// paths.
pub fn resolve_target(working_dir: &Path, path: &Path) -> PathBuf
{
    let resolved = if path.is_absolute()
    {
        path.to_path_buf()
    }
    else
    {
        working_dir.join(path)
    };
    resolved.clean()
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs;

    #[test]
    fn finds_marker_in_start_dir()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();

        // Act
        let result = Vmr::find(tmp.path()).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn normalizes_found_vmr_path()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("child")).unwrap();
        let working_dir = tmp.path().join("child").join("..");

        // Act
        let result = Vmr::find(&working_dir).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn finds_marker_in_ancestor()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("repo/src");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();

        // Act
        let result = Vmr::find(&nested).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn errors_when_marker_is_missing()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let err = Vmr::find(tmp.path()).unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("not a virtual monorepo"));
    }

    fn vmr_fixture() -> tempfile::TempDir
    {
        // Arrange fixture with Git and non-Git children
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("backend")).unwrap();
        fs::create_dir(tmp.path().join("backend/.git")).unwrap();
        fs::create_dir(tmp.path().join("frontend")).unwrap();
        fs::create_dir(tmp.path().join("frontend/.git")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        fs::write(tmp.path().join("README.md"), "vmr\n").unwrap();
        tmp
    }

    #[test]
    fn normalizes_paths_lexically_without_requiring_existence()
    {
        // Act
        let path = Path::new("/tmp/vmr/backend/../backend/deleted.rs").clean();

        // Assert
        assert_eq!(path, PathBuf::from("/tmp/vmr/backend/deleted.rs"));
    }

    #[test]
    fn resolves_relative_paths_against_working_dir_and_normalizes()
    {
        // Act
        let path = resolve_target(
            Path::new("/tmp/vmr/frontend"),
            Path::new("../backend/a.rs")
        );

        // Assert
        assert_eq!(path, PathBuf::from("/tmp/vmr/backend/a.rs"));
    }

    #[test]
    fn resolve_target_keeps_absolute_paths()
    {
        // Act
        let path = resolve_target(
            Path::new("/tmp/vmr/frontend"),
            Path::new("/tmp/vmr/backend/../backend/a.rs")
        );

        // Assert
        assert_eq!(path, PathBuf::from("/tmp/vmr/backend/a.rs"));
    }

    #[test]
    fn is_root_accepts_marker_directory_and_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path().join("main");
        fs::create_dir_all(main_root.join(".gitvmr")).unwrap();
        let worktree_root = tmp.path().join("worktree");
        fs::create_dir_all(&worktree_root).unwrap();
        fs::write(worktree_root.join(".gitvmr"), "").unwrap();
        let plain = tmp.path().join("plain");
        fs::create_dir_all(&plain).unwrap();

        // Assert
        assert!(Vmr::is_root(&main_root));
        assert!(Vmr::is_root(&worktree_root));
        assert!(!Vmr::is_root(&plain));
    }

    #[test]
    fn init_creates_marker_directory_with_config()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let outcome = Vmr::init(tmp.path()).unwrap();

        // Assert
        assert_eq!(outcome, InitOutcome::Created);
        assert!(tmp.path().join(".gitvmr").is_dir());
        assert!(tmp.path().join(".gitvmr/config").is_file());
    }

    #[test]
    fn init_detects_an_existing_root()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        Vmr::init(tmp.path()).unwrap();
        let config =
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap();

        // Act
        let outcome = Vmr::init(tmp.path()).unwrap();

        // Assert
        assert_eq!(outcome, InitOutcome::Reinitialized);
        assert_eq!(
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap(),
            config
        );
    }

    #[test]
    fn create_worktree_root_materializes_directory_and_marker_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("feature");

        // Act
        Vmr::create_worktree_root(&target).unwrap();

        // Assert
        assert!(target.is_dir());
        assert!(target.join(".gitvmr").is_file());
        assert!(Vmr::is_root(&target));
    }

    #[test]
    fn remove_worktree_root_removes_marker_file_and_empty_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("feature");
        Vmr::create_worktree_root(&target).unwrap();

        // Act
        Vmr::remove_worktree_root(&target).unwrap();

        // Assert
        assert!(!target.exists());
    }

    #[test]
    fn remove_worktree_root_removes_marker_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("feature");
        fs::create_dir_all(target.join(".gitvmr")).unwrap();

        // Act
        Vmr::remove_worktree_root(&target).unwrap();

        // Assert
        assert!(!target.exists());
    }

    #[test]
    fn remove_worktree_root_keeps_non_empty_directories()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("feature");
        Vmr::create_worktree_root(&target).unwrap();
        fs::write(target.join("keep.txt"), "contents").unwrap();

        // Act
        Vmr::remove_worktree_root(&target).unwrap();

        // Assert
        assert!(!target.join(".gitvmr").exists());
        assert!(target.join("keep.txt").exists());
    }

    #[test]
    fn remove_worktree_root_tolerates_missing_target()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let result = Vmr::remove_worktree_root(&tmp.path().join("missing"));

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn routes_repo_root_to_dot()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let routed = vmr
            .route_path(&vmr.repos().unwrap(), &tmp.path().join("backend"))
            .unwrap();

        // Assert
        assert_eq!(routed, vec![(
            repo(tmp.path(), "backend"),
            PathBuf::from(".")
        )]);
    }

    #[test]
    fn routes_child_path_to_repo_relative_path()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let routed = vmr
            .route_path(
                &vmr.repos().unwrap(),
                &tmp.path().join("backend/src/main.rs")
            )
            .unwrap();

        // Assert
        assert_eq!(routed, vec![(
            repo(tmp.path(), "backend"),
            PathBuf::from("src/main.rs")
        )]);
    }

    #[test]
    fn expands_vmr_root_to_immediate_child_git_repos()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let routed = vmr.route_path(&vmr.repos().unwrap(), tmp.path()).unwrap();

        // Assert
        assert_eq!(routed, vec![
            (repo(tmp.path(), "backend"), PathBuf::from(".")),
            (repo(tmp.path(), "frontend"), PathBuf::from("."))
        ]);
    }

    #[test]
    fn vmr_repos_skips_non_git_dirs_and_sorts_by_name()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("zeta")).unwrap();
        fs::create_dir(tmp.path().join("zeta/.git")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        fs::create_dir(tmp.path().join("alpha")).unwrap();
        fs::create_dir(tmp.path().join("alpha/.git")).unwrap();
        let vmr = Vmr::find(tmp.path()).unwrap();

        // Act
        let repos = vmr.repos().unwrap();

        // Assert
        assert_eq!(repos, vec![
            repo(tmp.path(), "alpha"),
            repo(tmp.path(), "zeta")
        ]);
    }

    #[test]
    fn rejects_paths_outside_vmr_root()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let err = vmr
            .route_path(
                &vmr.repos().unwrap(),
                &tmp.path().join("../outside.txt").clean()
            )
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("outside virtual monorepo"));
    }

    #[test]
    fn rejects_vmr_metadata_paths()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let err = vmr
            .route_path(
                &vmr.repos().unwrap(),
                &tmp.path().join(".gitvmr/config")
            )
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("metadata"));
    }

    #[test]
    fn rejects_non_git_child_paths()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let err = vmr
            .route_path(
                &vmr.repos().unwrap(),
                &tmp.path().join("docs/readme.md")
            )
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("child Git repository"));
    }

    #[test]
    fn rejects_root_files()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let err = vmr
            .route_path(&vmr.repos().unwrap(), &tmp.path().join("README.md"))
            .expect_err("path should fail");

        // Assert
        assert!(format!("{err:#}").contains("child Git repository"));
    }

    #[test]
    fn groups_routes_by_repository()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let routed = vmr
            .route_paths(&vmr.repos().unwrap(), tmp.path(), &[
                PathBuf::from("backend/src/main.rs"),
                PathBuf::from("backend/lib.rs"),
                PathBuf::from("frontend/app.rs")
            ])
            .unwrap();

        // Assert
        assert_eq!(routed.get(&repo(tmp.path(), "backend")).unwrap(), &vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("lib.rs")
        ]);
        assert_eq!(routed.get(&repo(tmp.path(), "frontend")).unwrap(), &vec![
            PathBuf::from("app.rs")
        ]);
    }

    #[test]
    fn routes_single_path_without_vmr_root_expansion()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let routed = vmr
            .route_single_path(
                &vmr.repos().unwrap(),
                tmp.path(),
                Path::new("backend/src/main.rs")
            )
            .expect("path should route");

        // Assert
        assert_eq!(
            routed,
            (repo(tmp.path(), "backend"), PathBuf::from("src/main.rs"))
        );
    }

    #[test]
    fn route_single_path_rejects_vmr_root()
    {
        // Arrange
        let tmp = vmr_fixture();
        let vmr = Vmr::new(tmp.path());

        // Act
        let err = vmr
            .route_single_path(
                &vmr.repos().unwrap(),
                tmp.path(),
                Path::new(".")
            )
            .expect_err("VMR root should fail");

        // Assert
        assert!(format!("{err:#}").contains("VMR root aggregate"));
    }

    fn repo(root: &Path, name: &str) -> Repo
    {
        Repo { name: name.to_owned(), path: root.join(name) }
    }
}
