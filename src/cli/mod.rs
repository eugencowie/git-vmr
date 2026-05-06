mod add;
mod init;
mod mv;
mod restore;
mod rm;
mod status;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Subcommand)]
enum Command
{
    /// Create an empty virtual monorepo or reinitialize an existing one
    Init
    {
        /// If you provide a directory, the command is run inside it. If this
        /// directory does not exist, it will be created
        #[arg(value_name = "directory")]
        directory: Option<PathBuf>
    },

    /// Add file contents to the index
    Add
    {
        /// Files to add content from
        #[arg(required = true, num_args = 1.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Move or rename a file, a directory, or a symlink
    Mv
    {
        /// File to move
        #[arg(value_name = "source")]
        source: PathBuf,

        /// Destination path
        #[arg(value_name = "destination")]
        destination: PathBuf
    },

    /// Restore working tree files
    Restore
    {
        /// Restore the working tree
        #[arg(long)]
        worktree: bool,

        /// Restore the index
        #[arg(long)]
        staged: bool,

        /// Files to restore
        #[arg(required = true, num_args = 1.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Remove files from the working tree and from the index
    Rm
    {
        /// Allow recursive removal when a leading directory name is given
        #[arg(short)]
        recursive: bool,

        /// Files to remove
        #[arg(required = true, num_args = 1.., value_name = "pathspec")]
        paths: Vec<PathBuf>
    },

    /// Show the working tree status
    Status
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
            Command::Init { directory } =>
                init::init(&working_dir, directory.as_deref()),
            Command::Add { paths } => add::add(&working_dir, &paths),
            Command::Mv { source, destination } =>
                mv::mv(&working_dir, &source, &destination),
            Command::Restore { paths, staged, worktree } =>
                restore::restore(&working_dir, &paths, worktree, staged),
            Command::Rm { paths, recursive } =>
                rm::rm(&working_dir, &paths, recursive),
            Command::Status => status::status(&working_dir)
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
    use std::path::Path;

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
    fn parses_init_directory_argument()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "init", "project"]);

        // Assert
        match cli.command
        {
            Command::Init { directory } =>
            {
                assert_eq!(directory.as_deref(), Some(Path::new("project")));
            }
            _ => panic!("expected init command")
        }
    }

    #[test]
    fn parses_working_dir_with_init_directory_argument()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "-C",
            tmp.path().to_str().unwrap(),
            "init",
            "project"
        ]);

        // Assert
        assert_eq!(cli.working_dir.as_deref(), Some(tmp.path()));
        match cli.command
        {
            Command::Init { directory } =>
            {
                assert_eq!(directory.as_deref(), Some(Path::new("project")));
            }
            _ => panic!("expected init command")
        }
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
