mod manifest;
mod repo;

use anyhow::{Context, Result, bail};
pub use manifest::{Declaration, Manifest, ManifestEditor, validate_path};
use path_clean::PathClean;
pub use repo::Repo;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The manifest's file name, at the VMR root.
pub const MANIFEST: &str = ".gitvmr";

/// The root repo's `.gitignore`, or nothing if it has none.
pub fn read_gitignore(root: &Path) -> String
{
    fs::read_to_string(root.join(".gitignore")).unwrap_or_default()
}

/// Whether `.gitignore` text has the exact anchored line git-vmr writes to
/// ignore the child repo at `path`.
pub fn has_ignore_line(gitignore: &str, path: &str) -> bool
{
    let line = format!("/{path}/");
    gitignore.lines().any(|candidate| candidate.trim_end() == line)
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

    /// Discovers the VMR that owns `working_dir`: the nearest directory,
    /// walking up, where a `.gitvmr` manifest file sits beside `.git` (a
    /// directory, or a file in a linked worktree). A filesystem check only:
    /// a root repo on a branch without a manifest is indistinguishable from
    /// any other Git repository.
    pub fn find(working_dir: &Path) -> Result<Vmr>
    {
        for dir in working_dir.clean().ancestors()
        {
            let manifest = dir.join(MANIFEST);
            if manifest.is_dir()
            {
                bail!(
                    "fatal: '{}' is an old-style virtual monorepo\n\
                     hint: run 'git vmr init' there to convert it",
                    dir.display()
                );
            }

            if manifest.is_file()
            {
                if dir.join(".git").exists()
                {
                    return Ok(Vmr::new(dir));
                }

                Vmr::reject_old_worktree_root(dir)?;
            }
        }

        bail!(
            "fatal: not a virtual monorepo (or any of the parent \
             directories): .gitvmr\n\
             hint: if this is a root repo, the checked-out branch may not \
             have a .gitvmr manifest"
        )
    }

    /// Rejects an old-style worktree root: an empty `.gitvmr` file with no
    /// `.git` beside it.
    pub fn reject_old_worktree_root(dir: &Path) -> Result<()>
    {
        let empty_manifest = fs::metadata(dir.join(MANIFEST))
            .is_ok_and(|meta| meta.is_file() && meta.len() == 0);
        if empty_manifest && !dir.join(".git").exists()
        {
            bail!(
                "fatal: '{}' is an old-style worktree root\n\
                 hint: remove its child worktrees with 'git worktree remove' \
                 in each repo, then recreate it with 'git vmr worktree add'",
                dir.display()
            );
        }

        Ok(())
    }

    /// Reads and validates the manifest from the working tree.
    pub fn manifest(&self) -> Result<Manifest>
    {
        let path = self.path.join(MANIFEST);
        let text = fs::read_to_string(&path).with_context(|| {
            format!("fatal: failed to read '{}'", path.display())
        })?;

        Manifest::parse(&text).with_context(|| {
            format!("fatal: invalid manifest '{}'", path.display())
        })
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
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs;

    /// A VMR root: a root repo (a `.git` directory, enough for discovery)
    /// carrying a manifest.
    fn root_repo(dir: &Path)
    {
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::write(dir.join(".gitvmr"), "").unwrap();
    }

    #[test]
    fn finds_root_repo_with_manifest_in_start_dir()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path());

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
        root_repo(tmp.path());
        fs::create_dir(tmp.path().join("child")).unwrap();
        let working_dir = tmp.path().join("child").join("..");

        // Act
        let result = Vmr::find(&working_dir).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn finds_root_repo_from_inside_a_child_repo()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path());
        let nested = tmp.path().join("repo/src");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir(tmp.path().join("repo/.git")).unwrap();

        // Act
        let result = Vmr::find(&nested).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn accepts_a_git_file_beside_the_manifest()
    {
        // Arrange: a linked worktree's `.git` is a file
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".git"), "gitdir: elsewhere\n").unwrap();
        fs::write(tmp.path().join(".gitvmr"), "").unwrap();

        // Act
        let result = Vmr::find(tmp.path()).unwrap();

        // Assert
        assert_eq!(result.path, tmp.path());
    }

    #[test]
    fn nearest_vmr_wins()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path());
        let inner = tmp.path().join("inner");
        root_repo(&inner);

        // Act
        let result = Vmr::find(&inner).unwrap();

        // Assert
        assert_eq!(result.path, inner);
    }

    #[test]
    fn errors_when_no_root_repo_carries_a_manifest()
    {
        // Arrange: a Git repository without a manifest is not a VMR
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();

        // Act
        let err = Vmr::find(tmp.path()).unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            "fatal: not a virtual monorepo (or any of the parent \
             directories): .gitvmr\n\
             hint: if this is a root repo, the checked-out branch may not \
             have a .gitvmr manifest"
        );
    }

    #[test]
    fn errors_on_an_old_style_vmr()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".gitvmr")).unwrap();
        let nested = tmp.path().join("backend");
        fs::create_dir(&nested).unwrap();

        // Act
        let err = Vmr::find(&nested).unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            format!(
                "fatal: '{}' is an old-style virtual monorepo\n\
                 hint: run 'git vmr init' there to convert it",
                tmp.path().display()
            )
        );
    }

    #[test]
    fn errors_on_an_old_style_worktree_root()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".gitvmr"), "").unwrap();

        // Act
        let err = Vmr::find(tmp.path()).unwrap_err();

        // Assert
        assert_eq!(
            format!("{err:#}"),
            format!(
                "fatal: '{}' is an old-style worktree root\n\
                 hint: remove its child worktrees with 'git worktree remove' \
                 in each repo, then recreate it with 'git vmr worktree add'",
                tmp.path().display()
            )
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
}
