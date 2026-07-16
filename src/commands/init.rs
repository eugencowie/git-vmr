use crate::cli::CliContext;
use crate::render::Rendered;
use crate::vmr::{InitOutcome, Vmr};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::PathBuf;

/// Create an empty virtual monorepo or reinitialize an existing one
#[derive(clap::Args)]
pub struct InitArgs
{
    /// If you provide a directory, the command is run inside it. If this
    /// directory does not exist, it will be created
    #[arg(value_name = "directory")]
    pub directory: Option<PathBuf>
}

pub fn run(context: &CliContext, args: &InitArgs) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    // Resolve init target directory
    let target_dir = match args.directory.as_deref()
    {
        // Resolve provided directory against the effective working directory
        Some(directory) => working_dir.join(directory),

        // If none provided, use the effective working directory
        None => working_dir.to_path_buf()
    };

    // Ensure the target path is a directory
    if target_dir.exists() && !target_dir.is_dir()
    {
        bail!(
            "fatal: cannot initialize '{}': Not a directory",
            target_dir.display()
        );
    }

    // Create target directory
    fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "fatal: failed to create init directory '{}'",
            target_dir.display()
        )
    })?;

    let message = match Vmr::init(&target_dir)?
    {
        InitOutcome::Created =>
            format!("Created .gitvmr in {}\n", target_dir.display()),
        InitOutcome::Reinitialized => format!(
            "Reinitialized existing virtual monorepo in {}/\n",
            target_dir.join(".gitvmr").display()
        )
    };

    Ok(message.into())
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::config::Config;
    use crate::test_support::cli_context;
    use std::fs;
    use std::path::Path;

    #[test]
    fn creates_config_directory_and_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        run(&cli_context(tmp.path()), &InitArgs { directory: None }).unwrap();

        // Assert
        assert!(tmp.path().join(".gitvmr").exists());
        assert!(tmp.path().join(".gitvmr/config").exists());
    }

    #[test]
    fn creates_config_in_absolute_target_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("project");

        // Act
        run(&cli_context(Path::new("/ignored")), &InitArgs {
            directory: Some(target.clone())
        })
        .unwrap();

        // Assert
        assert!(target.join(".gitvmr/config").is_file());
    }

    #[test]
    fn writes_default_config()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        run(&cli_context(tmp.path()), &InitArgs { directory: None }).unwrap();

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
        run(&cli_context(tmp.path()), &InitArgs { directory: None }).unwrap();
        let first =
            fs::read_to_string(tmp.path().join(".gitvmr/config")).unwrap();

        // Act
        run(&cli_context(tmp.path()), &InitArgs { directory: None }).unwrap();

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
        run(&cli_context(tmp.path()), &InitArgs { directory: None }).unwrap();

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
        let err = run(&cli_context(tmp.path()), &InitArgs {
            directory: Some(PathBuf::from("file"))
        })
        .unwrap_err();

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
        run(&cli_context(tmp.path()), &InitArgs {
            directory: Some(PathBuf::from("project"))
        })
        .unwrap();

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
        run(&cli_context(tmp.path()), &InitArgs {
            directory: Some(PathBuf::from("project"))
        })
        .unwrap();

        // Assert
        assert!(target.join(".gitvmr/config").is_file());
        assert!(!tmp.path().join(".gitvmr").exists());
    }
}
