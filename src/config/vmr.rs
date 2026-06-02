use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub fn find_vmr_root(start: &Path) -> Result<PathBuf>
{
    // Search start directory and ancestors
    for parent in start.ancestors()
    {
        if parent.join(".gitvmr").exists()
        {
            return Ok(parent.to_path_buf());
        }
    }

    // Report missing VMR root
    bail!("not a virtual monorepo (or any of the parent directories): .gitvmr")
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
        let result = find_vmr_root(tmp.path()).unwrap();

        // Assert
        assert_eq!(result, tmp.path());
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
        let result = find_vmr_root(&nested).unwrap();

        // Assert
        assert_eq!(result, tmp.path());
    }

    #[test]
    fn errors_when_marker_is_missing()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let err = find_vmr_root(tmp.path()).unwrap_err();

        // Assert
        assert!(
            format!("{err:#}").contains("not a virtual monorepo"),
            "unexpected error: {err:#}"
        );
    }
}
