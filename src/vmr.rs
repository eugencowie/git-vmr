mod repo;

use crate::config::Config;
use crate::store::FileStore;
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
pub use repo::Repo;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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

    fn repo(root: &Path, name: &str) -> Repo
    {
        Repo { name: name.to_owned(), path: root.join(name) }
    }
}
