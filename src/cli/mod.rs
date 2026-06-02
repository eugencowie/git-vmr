mod init;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Subcommand)]
enum Command
{
    /// Create an empty virtual monorepo or reinitialize an existing one
    Init
}

#[derive(Parser)]
#[command(name = "git-vmr", version, disable_version_flag = true)]
pub struct Cli
{
    /// Run as if git-vmr was started in <path> instead of the current working
    /// directory
    #[arg(short = 'C', value_name = "path")]
    working_dir: Option<PathBuf>,

    /// Print version
    #[arg(short, long, action = ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    command: Command
}

impl Cli
{
    pub fn run(self) -> Result<()>
    {
        // Get working directory
        let working_dir = self.get_working_dir()?;

        // Run command
        match self.command
        {
            Command::Init => init::init(&working_dir)
        }
    }

    fn get_working_dir(&self) -> Result<PathBuf>
    {
        match &self.working_dir
        {
            // Parse working directory from argument
            Some(working_dir) =>
            {
                // Get absolute path
                let absolute_path =
                    working_dir.canonicalize().with_context(|| {
                        format!("cannot change to '{}'", working_dir.display())
                    })?;

                // Ensure the path is a directory
                if !absolute_path.is_dir()
                {
                    bail!(
                        "cannot change to '{}': Not a directory",
                        working_dir.display()
                    );
                }

                Ok(absolute_path)
            }

            // If none provided, use current working directory
            None =>
                env::current_dir().context("failed to get current directory"),
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use clap::Parser;
    use std::fs;

    #[test]
    fn parses_working_dir_argument()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "-C",
            tmp.path().to_str().unwrap(),
            "init"
        ]);

        // Assert
        assert_eq!(cli.working_dir.as_deref(), Some(tmp.path()));
    }

    #[test]
    fn init_uses_canonicalized_working_dir_argument()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let non_canonical = nested.join("..").join("nested");

        // Act
        Cli::parse_from([
            "git-vmr",
            "-C",
            non_canonical.to_str().unwrap(),
            "init"
        ])
        .run()
        .unwrap();

        // Assert
        assert!(nested.canonicalize().unwrap().join(".gitvmr/config").exists());
    }

    #[test]
    fn errors_on_nonexistent_working_dir_path()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("missing");

        // Act
        let err = Cli::parse_from([
            "git-vmr",
            "-C",
            missing.to_str().unwrap(),
            "init"
        ])
        .run()
        .unwrap_err();

        // Assert
        let msg = format!("{err:#}");
        assert!(msg.contains("cannot change to"), "unexpected error: {msg}");
        assert!(!tmp.path().join(".gitvmr").exists());
    }

    #[test]
    fn errors_on_file_as_working_dir_path()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("file");
        fs::write(&file, "").unwrap();

        // Act
        let err =
            Cli::parse_from(["git-vmr", "-C", file.to_str().unwrap(), "init"])
                .run()
                .unwrap_err();

        // Assert
        let msg = format!("{err:#}");
        assert!(msg.contains("Not a directory"), "unexpected error: {msg}");
        assert!(!file.join(".gitvmr").exists());
    }
}
