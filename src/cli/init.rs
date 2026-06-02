use crate::config::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn init(working_dir: &Path) -> Result<()>
{
    // Construct path to config file
    let vmr_dir = working_dir.join(".gitvmr");
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
    println!("Created .gitvmr in {}", working_dir.display());

    Ok(())
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
        init(tmp.path()).unwrap();

        // Assert
        assert!(tmp.path().join(".gitvmr").exists());
        assert!(tmp.path().join(".gitvmr/config").exists());
    }

    #[test]
    fn writes_default_config()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        init(tmp.path()).unwrap();

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
        init(tmp.path()).unwrap();
        let first =
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap();

        // Act
        init(tmp.path()).unwrap();

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
        init(tmp.path()).unwrap();

        // Assert
        let contents = fs::read_to_string(config_dir.join("config")).unwrap();
        assert_eq!(contents, "custom content");
    }
}
