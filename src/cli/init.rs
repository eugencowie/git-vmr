use crate::config::Config;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

pub fn init(working_dir: &Path, directory: Option<&Path>) -> Result<()>
{
    // Resolve init target directory
    let target_dir = resolve_target_dir(working_dir, directory);

    // Ensure the target path is a directory
    if target_dir.exists() && !target_dir.is_dir()
    {
        bail!("cannot initialize '{}': Not a directory", target_dir.display());
    }

    // Create target directory
    fs::create_dir_all(&target_dir).with_context(|| {
        format!("failed to create init directory '{}'", target_dir.display())
    })?;

    // Construct path to config file
    let vmr_dir = target_dir.join(".gitvmr");
    let config_path = vmr_dir.join("config");

    // Check if config file already exists
    if config_path.is_file()
    {
        println!(
            "Reinitialized existing virtual monorepo in {}/",
            vmr_dir.display()
        );
        return Ok(());
    }

    // Create VMR directory
    fs::create_dir_all(&vmr_dir)
        .context("failed to create .gitvmr directory")?;

    // Create config file
    let config = toml::to_string(&Config::default())
        .context("failed to serialize config")?;
    fs::write(&config_path, config)
        .context("failed to write .gitvmr/config")?;
    println!("Created .gitvmr in {}", target_dir.display());

    Ok(())
}

fn resolve_target_dir(working_dir: &Path, directory: Option<&Path>) -> PathBuf
{
    match directory
    {
        // Resolve provided directory against the effective working directory
        Some(directory) => working_dir.join(directory),

        // If none provided, use the effective working directory
        None => working_dir.to_path_buf()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs;

    #[test]
    fn creates_config_directory_and_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        init(tmp.path(), None).unwrap();

        // Assert
        assert!(tmp.path().join(".gitvmr").exists());
        assert!(tmp.path().join(".gitvmr/config").exists());
    }

    #[test]
    fn resolves_missing_directory_argument_to_working_dir()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let target = resolve_target_dir(tmp.path(), None);

        // Assert
        assert_eq!(target, tmp.path());
    }

    #[test]
    fn resolves_relative_directory_argument_against_working_dir()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let target = resolve_target_dir(tmp.path(), Some(Path::new("project")));

        // Assert
        assert_eq!(target, tmp.path().join("project"));
    }

    #[test]
    fn preserves_absolute_directory_argument()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("project");

        // Act
        let resolved = resolve_target_dir(Path::new("/ignored"), Some(&target));

        // Assert
        assert_eq!(resolved, target);
    }

    #[test]
    fn writes_default_config()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        init(tmp.path(), None).unwrap();

        // Assert
        let contents =
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap();
        let expected = toml::to_string(&Config::default()).unwrap();
        assert_eq!(contents, expected);
    }

    #[test]
    fn is_idempotent()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path(), None).unwrap();
        let first =
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap();

        // Act
        init(tmp.path(), None).unwrap();

        // Assert
        let second =
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn does_not_overwrite_existing_config()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join(".gitvmr");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config"), "custom content").unwrap();

        // Act
        init(tmp.path(), None).unwrap();

        // Assert
        let contents = fs::read_to_string(config_dir.join("config")).unwrap();
        assert_eq!(contents, "custom content");
    }

    #[test]
    fn errors_when_target_exists_as_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("file");
        fs::write(&file, "").unwrap();

        // Act
        let err = init(tmp.path(), Some(Path::new("file"))).unwrap_err();

        // Assert
        let msg = format!("{err:#}");
        assert!(msg.contains("Not a directory"), "unexpected error: {msg}");
        assert!(!file.join(".gitvmr/config").exists());
    }

    #[test]
    fn creates_missing_target_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("project");

        // Act
        init(tmp.path(), Some(Path::new("project"))).unwrap();

        // Assert
        assert!(target.is_dir());
        assert!(target.join(".gitvmr/config").is_file());
    }

    #[test]
    fn creates_config_in_existing_target_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("project");
        fs::create_dir(&target).unwrap();

        // Act
        init(tmp.path(), Some(Path::new("project"))).unwrap();

        // Assert
        assert!(target.join(".gitvmr/config").is_file());
        assert!(!tmp.path().join(".gitvmr").exists());
    }
}
