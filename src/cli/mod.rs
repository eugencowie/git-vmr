mod add;
mod branch;
mod commit;
mod init;
mod merge;
mod mv;
mod restore;
mod rm;
mod status;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct AggregateError
{
    errors: Vec<anyhow::Error>
}

impl AggregateError
{
    pub(crate) fn new(errors: Vec<anyhow::Error>) -> Self
    {
        Self { errors }
    }

    pub(crate) fn errors(&self) -> &[anyhow::Error]
    {
        &self.errors
    }
}

impl std::fmt::Display for AggregateError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{} errors occurred", self.errors.len())
    }
}

impl std::error::Error for AggregateError {}

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
    Status,

    /// List or create branches
    Branch
    {
        /// Creates a new branch head named [branch-name] which points to the
        /// current HEAD
        #[arg(value_name = "branch-name")]
        branch_name: Option<String>
    },

    /// Record changes to the repositories
    Commit
    {
        /// Use <msg> as the commit message
        #[arg(short, long, required = true, value_name = "msg")]
        message: String
    },

    /// Join two or more development histories together
    Merge
    {
        /// Commits, usually other branch heads, to merge into our branch
        #[arg(required = true, value_name = "commit")]
        commit_ish: String
    }
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
            Command::Status => status::status(&working_dir),
            Command::Branch { branch_name } =>
                branch::branch(&working_dir, branch_name.as_deref()),
            Command::Commit { message } =>
                commit::commit(&working_dir, &message),
            Command::Merge { commit_ish } =>
                merge::merge(&working_dir, &commit_ish),
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

    #[test]
    fn parses_branch_without_branch_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "branch"]);

        // Assert
        match cli.command
        {
            Command::Branch { branch_name } =>
            {
                assert_eq!(branch_name, None);
            }
            _ => panic!("expected branch command")
        }
    }

    #[test]
    fn parses_branch_with_branch_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "branch", "feature/auth"]);

        // Assert
        match cli.command
        {
            Command::Branch { branch_name } =>
            {
                assert_eq!(branch_name.as_deref(), Some("feature/auth"));
            }
            _ => panic!("expected branch command")
        }
    }

    #[test]
    fn parses_commit_with_short_message()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "commit",
            "-m",
            "Implement new feature"
        ]);

        // Assert
        match cli.command
        {
            Command::Commit { message } =>
            {
                assert_eq!(message, "Implement new feature");
            }
            _ => panic!("expected commit command")
        }
    }

    #[test]
    fn parses_commit_with_long_message()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "commit",
            "--message",
            "Implement new feature"
        ]);

        // Assert
        match cli.command
        {
            Command::Commit { message } =>
            {
                assert_eq!(message, "Implement new feature");
            }
            _ => panic!("expected commit command")
        }
    }

    #[test]
    fn rejects_commit_without_message()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "commit"])
        {
            Ok(_) => panic!("expected commit parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn parses_merge_with_commit_ish()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "merge", "feature/auth"]);

        // Assert
        match cli.command
        {
            Command::Merge { commit_ish } =>
            {
                assert_eq!(commit_ish, "feature/auth");
            }
            _ => panic!("expected merge command")
        }
    }

    #[test]
    fn rejects_merge_without_commit_ish()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "merge"])
        {
            Ok(_) => panic!("expected merge parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }
}
