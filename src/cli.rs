mod add;
mod branch;
mod clone;
mod commit;
mod fetch;
mod init;
mod merge;
mod mv;
mod pull;
mod rebase;
mod restore;
mod rm;
mod status;
mod switch;
mod tag;

use crate::git::GitCommandResult;
use anyhow::{Context, Result, bail};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AggregateError
{
    errors: Vec<anyhow::Error>
}

impl AggregateError
{
    pub fn new(errors: Vec<anyhow::Error>) -> Self
    {
        Self { errors }
    }

    pub fn errors(&self) -> &[anyhow::Error]
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

fn print_results(results: Vec<GitCommandResult>) -> Result<()>
{
    let mut messages = Vec::new();
    let mut errors = Vec::new();

    for result in results
    {
        match result
        {
            Ok(Some(message)) => messages.push(message),
            Ok(None) =>
            {}
            Err(error) => errors.push(error)
        }
    }

    for message in messages
    {
        println!("{message}");
    }

    if !errors.is_empty()
    {
        return Err(AggregateError::new(errors).into());
    }

    Ok(())
}

#[derive(Subcommand)]
enum Command
{
    /// Clone a repository into a new directory
    Clone
    {
        /// The (possibly remote) <repository> to clone from
        #[arg(value_name = "repository")]
        repository: String,

        /// The name of a new directory to clone into
        #[arg(value_name = "directory")]
        directory: Option<PathBuf>
    },

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

    /// List, create, or delete branches
    Branch
    {
        /// Delete a branch. The branch must be fully merged in its upstream
        /// branch
        #[arg(
            short,
            long,
            conflicts_with = "force_delete",
            requires = "branch_name"
        )]
        delete: bool,

        /// Shortcut for `--delete --force`
        #[arg(
            short = 'D',
            conflicts_with = "delete",
            requires = "branch_name"
        )]
        force_delete: bool,

        /// In combination with `-d` (or `--delete`), allow deleting the branch
        /// irrespective of its merged status, or whether it even points to a
        /// valid commit
        #[arg(
            short,
            long,
            conflicts_with = "force_delete",
            requires_all = ["branch_name", "delete"]
        )]
        force: bool,

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
    },

    /// Reapply commits on top of another base tip
    Rebase
    {
        /// Upstream branch to compare against
        #[arg(required = true, value_name = "upstream")]
        upstream: String
    },

    /// Switch branches
    Switch
    {
        /// Branch to switch to
        #[arg(required = true, value_name = "branch")]
        branch_name: String
    },

    /// Create, list, delete or verify tags
    Tag
    {
        /// Delete existing tags with the given names
        #[arg(short, long, requires = "tag_name")]
        delete: bool,

        /// The name of the tag to create, delete, or describe
        #[arg(value_name = "tagname")]
        tag_name: Option<String>
    },

    /// Download objects and refs from another repository
    Fetch
    {
        /// The "remote" repository that is the source of a fetch or pull
        /// operation
        #[arg(value_name = "repository")]
        repository: Option<String>,

        /// Specifies which refs to fetch and which local refs to update
        #[arg(value_name = "refspec")]
        refspecs: Vec<String>
    },

    /// Fetch from and integrate with another repository or a local branch
    Pull
    {
        /// The "remote" repository to pull from
        #[arg(value_name = "repository")]
        repository: Option<String>,

        /// Which branch or other reference(s) to fetch and integrate into the
        /// current branch
        #[arg(value_name = "refspec")]
        refspecs: Vec<String>
    }
}

#[derive(Parser)]
#[command(name = "git-vmr", version, disable_version_flag = true)]
pub struct Cli
{
    #[arg(skip)]
    bin_name: String,

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
    pub fn parse() -> Self
    {
        Self::parse_from(env::args_os())
    }

    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone
    {
        Self::try_parse_from(itr).unwrap_or_else(|err| err.exit())
    }

    pub fn try_parse_from<I, T>(
        itr: I
    ) -> std::result::Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone
    {
        let mut command = Self::command();
        let mut matches = command.try_get_matches_from_mut(itr)?;
        let bin_name = command
            .get_bin_name()
            .unwrap_or_else(|| command.get_name())
            .to_owned();

        let mut cli = Self::from_arg_matches_mut(&mut matches)?;
        cli.bin_name = display_bin_name(&bin_name);

        Ok(cli)
    }

    pub fn run(self) -> Result<()>
    {
        // Get working directory
        let working_dir = self.get_working_dir()?;

        // Run command
        match self.command
        {
            Command::Clone { repository, directory } =>
                clone::clone(&working_dir, &repository, directory.as_deref()),
            Command::Init { directory } =>
                init::init(&working_dir, directory.as_deref()),
            Command::Add { paths } => add::add(&working_dir, &paths),
            Command::Mv { source, destination } =>
                mv::mv(&working_dir, &source, &destination),
            Command::Restore { paths, staged, worktree } =>
                restore::restore(&working_dir, &paths, worktree, staged),
            Command::Rm { paths, recursive } =>
                rm::rm(&working_dir, &paths, recursive),
            Command::Status => status::status(&self.bin_name, &working_dir),
            Command::Branch { delete, force_delete, force, branch_name } =>
                match (branch_name, delete, force_delete, force)
                {
                    (Some(branch_name), true, false, false) =>
                        branch::delete(&working_dir, &branch_name),
                    (Some(branch_name), false, true, false) =>
                        branch::force_delete(&working_dir, &branch_name),
                    (Some(branch_name), true, false, true) =>
                        branch::force_delete(&working_dir, &branch_name),
                    (Some(branch_name), false, false, false) =>
                        branch::branch(&working_dir, &branch_name),
                    (None, false, false, false) =>
                        branch::branches(&working_dir),
                    _ => unreachable!()
                },
            Command::Commit { message } =>
                commit::commit(&working_dir, &message),
            Command::Merge { commit_ish } =>
                merge::merge(&working_dir, &commit_ish),
            Command::Rebase { upstream } =>
                rebase::rebase(&working_dir, &upstream),
            Command::Switch { branch_name } =>
                switch::switch(&working_dir, &branch_name),
            Command::Tag { delete, tag_name } => match (tag_name, delete)
            {
                (Some(tag_name), true) => tag::delete(&working_dir, &tag_name),
                (Some(tag_name), false) => tag::create(&working_dir, &tag_name),
                (None, false) => tag::tag(&working_dir),
                _ => unreachable!()
            },
            Command::Fetch { repository, refspecs } =>
                fetch::fetch(&working_dir, repository.as_deref(), &refspecs),
            Command::Pull { repository, refspecs } =>
                pull::pull(&working_dir, repository.as_deref(), &refspecs),
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

fn display_bin_name(bin_name: &str) -> String
{
    match bin_name
    {
        "git-vmr" => "git vmr".to_owned(),
        _ => bin_name.to_owned()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
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
    fn captures_invoked_command_name()
    {
        // Act
        let cli = Cli::parse_from(["vv", "status"]);

        // Assert
        assert_eq!(cli.bin_name, "vv");
    }

    #[test]
    fn maps_canonical_binary_name_to_git_subcommand()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "status"]);

        // Assert
        assert_eq!(cli.bin_name, "git vmr");
    }

    #[test]
    fn captures_invoked_command_file_name()
    {
        // Act
        let cli = Cli::parse_from(["/usr/local/bin/vv", "status"]);

        // Assert
        assert_eq!(cli.bin_name, "vv");
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
    fn parses_clone_repository_argument()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "clone",
            "https://example.com/repo.git"
        ]);

        // Assert
        match cli.command
        {
            Command::Clone { repository, directory } =>
            {
                assert_eq!(repository, "https://example.com/repo.git");
                assert_eq!(directory, None);
            }
            _ => panic!("expected clone command")
        }
    }

    #[test]
    fn parses_clone_repository_and_directory_arguments()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "clone",
            "https://example.com/repo.git",
            "copy"
        ]);

        // Assert
        match cli.command
        {
            Command::Clone { repository, directory } =>
            {
                assert_eq!(repository, "https://example.com/repo.git");
                assert_eq!(directory.as_deref(), Some(Path::new("copy")));
            }
            _ => panic!("expected clone command")
        }
    }

    #[test]
    fn parses_working_dir_with_clone_arguments()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "-C",
            tmp.path().to_str().unwrap(),
            "clone",
            "https://example.com/repo.git",
            "copy"
        ]);

        // Assert
        assert_eq!(cli.working_dir.as_deref(), Some(tmp.path()));
        match cli.command
        {
            Command::Clone { repository, directory } =>
            {
                assert_eq!(repository, "https://example.com/repo.git");
                assert_eq!(directory.as_deref(), Some(Path::new("copy")));
            }
            _ => panic!("expected clone command")
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
            Command::Branch { delete, force_delete, force, branch_name } =>
            {
                assert!(!delete);
                assert!(!force_delete);
                assert!(!force);
                assert_eq!(branch_name, None);
            }
            _ => panic!("expected branch command")
        }
    }

    #[test]
    fn parses_tag_without_arguments()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "tag"]);

        // Assert
        match cli.command
        {
            Command::Tag { delete, tag_name } =>
            {
                assert!(!delete);
                assert_eq!(tag_name, None);
            }
            _ => panic!("expected tag command")
        }
    }

    #[test]
    fn parses_tag_with_tag_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "tag", "v1.0.0"]);

        // Assert
        match cli.command
        {
            Command::Tag { delete, tag_name } =>
            {
                assert!(!delete);
                assert_eq!(tag_name.as_deref(), Some("v1.0.0"));
            }
            _ => panic!("expected tag command")
        }
    }

    #[test]
    fn parses_tag_delete_with_tag_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "tag", "-d", "v1.0.0"]);

        // Assert
        match cli.command
        {
            Command::Tag { delete, tag_name } =>
            {
                assert!(delete);
                assert_eq!(tag_name.as_deref(), Some("v1.0.0"));
            }
            _ => panic!("expected tag command")
        }
    }

    #[test]
    fn parses_tag_long_delete_with_tag_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "tag", "--delete", "v1.0.0"]);

        // Assert
        match cli.command
        {
            Command::Tag { delete, tag_name } =>
            {
                assert!(delete);
                assert_eq!(tag_name.as_deref(), Some("v1.0.0"));
            }
            _ => panic!("expected tag command")
        }
    }

    #[test]
    fn tag_delete_requires_tag_name()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "tag", "-d"])
        {
            Ok(_) => panic!("expected tag parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_tag_extra_operand()
    {
        // Act
        let err =
            match Cli::try_parse_from(["git-vmr", "tag", "v1.0.0", "HEAD~1"])
            {
                Ok(_) => panic!("expected tag parse to fail"),
                Err(err) => err
            };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_tag_annotate_flag()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "tag", "-a", "v1.0.0"])
        {
            Ok(_) => panic!("expected tag parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_tag_force_flag()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "tag", "-f", "v1.0.0"])
        {
            Ok(_) => panic!("expected tag parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_tag_multi_delete()
    {
        // Act
        let err = match Cli::try_parse_from([
            "git-vmr", "tag", "-d", "v1.0.0", "v1.1.0"
        ])
        {
            Ok(_) => panic!("expected tag parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_tag_filter_flag()
    {
        // Act
        let err =
            match Cli::try_parse_from(["git-vmr", "tag", "--contains", "HEAD"])
            {
                Ok(_) => panic!("expected tag parse to fail"),
                Err(err) => err
            };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn parses_branch_with_branch_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "branch", "feature/auth"]);

        // Assert
        match cli.command
        {
            Command::Branch { delete, force_delete, force, branch_name } =>
            {
                assert!(!delete);
                assert!(!force_delete);
                assert!(!force);
                assert_eq!(branch_name.as_deref(), Some("feature/auth"));
            }
            _ => panic!("expected branch command")
        }
    }

    #[test]
    fn parses_branch_delete_with_branch_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "branch", "-d", "feature/auth"]);

        // Assert
        match cli.command
        {
            Command::Branch { delete, force_delete, force, branch_name } =>
            {
                assert!(delete);
                assert!(!force_delete);
                assert!(!force);
                assert_eq!(branch_name.as_deref(), Some("feature/auth"));
            }
            _ => panic!("expected branch command")
        }
    }

    #[test]
    fn parses_branch_force_delete_with_branch_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "branch", "-D", "feature/auth"]);

        // Assert
        match cli.command
        {
            Command::Branch { delete, force_delete, force, branch_name } =>
            {
                assert!(!delete);
                assert!(force_delete);
                assert!(!force);
                assert_eq!(branch_name.as_deref(), Some("feature/auth"));
            }
            _ => panic!("expected branch command")
        }
    }

    #[test]
    fn branch_delete_requires_branch_name()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "branch", "-d"])
        {
            Ok(_) => panic!("expected parse error"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn branch_delete_flags_conflict()
    {
        // Act
        let err = match Cli::try_parse_from([
            "git-vmr",
            "branch",
            "-d",
            "-D",
            "feature/auth"
        ])
        {
            Ok(_) => panic!("expected parse error"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn branch_force_requires_delete()
    {
        // Act
        let err = match Cli::try_parse_from([
            "git-vmr",
            "branch",
            "-f",
            "feature/auth"
        ])
        {
            Ok(_) => panic!("expected parse error"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
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
    fn parses_fetch_without_arguments()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "fetch"]);

        // Assert
        match cli.command
        {
            Command::Fetch { repository, refspecs } =>
            {
                assert_eq!(repository, None);
                assert!(refspecs.is_empty());
            }
            _ => panic!("expected fetch command")
        }
    }

    #[test]
    fn parses_fetch_with_repository_argument()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "fetch", "origin"]);

        // Assert
        match cli.command
        {
            Command::Fetch { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert!(refspecs.is_empty());
            }
            _ => panic!("expected fetch command")
        }
    }

    #[test]
    fn parses_fetch_with_repository_and_single_refspec()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "fetch", "origin", "main"]);

        // Assert
        match cli.command
        {
            Command::Fetch { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert_eq!(refspecs, ["main"]);
            }
            _ => panic!("expected fetch command")
        }
    }

    #[test]
    fn parses_fetch_with_repository_and_multiple_refspecs()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "fetch",
            "origin",
            "main",
            "release:release"
        ]);

        // Assert
        match cli.command
        {
            Command::Fetch { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert_eq!(refspecs, ["main", "release:release"]);
            }
            _ => panic!("expected fetch command")
        }
    }

    #[test]
    fn parses_working_dir_with_fetch_arguments()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "-C",
            tmp.path().to_str().unwrap(),
            "fetch",
            "origin",
            "main"
        ]);

        // Assert
        assert_eq!(cli.working_dir.as_deref(), Some(tmp.path()));
        match cli.command
        {
            Command::Fetch { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert_eq!(refspecs, ["main"]);
            }
            _ => panic!("expected fetch command")
        }
    }

    #[test]
    fn parses_pull_without_arguments()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "pull"]);

        // Assert
        match cli.command
        {
            Command::Pull { repository, refspecs } =>
            {
                assert_eq!(repository, None);
                assert!(refspecs.is_empty());
            }
            _ => panic!("expected pull command")
        }
    }

    #[test]
    fn parses_pull_with_repository_argument()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "pull", "origin"]);

        // Assert
        match cli.command
        {
            Command::Pull { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert!(refspecs.is_empty());
            }
            _ => panic!("expected pull command")
        }
    }

    #[test]
    fn parses_pull_with_repository_and_single_refspec()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "pull", "origin", "main"]);

        // Assert
        match cli.command
        {
            Command::Pull { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert_eq!(refspecs, ["main"]);
            }
            _ => panic!("expected pull command")
        }
    }

    #[test]
    fn parses_pull_with_repository_and_multiple_refspecs()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "pull", "origin", "main", "release"]);

        // Assert
        match cli.command
        {
            Command::Pull { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert_eq!(refspecs, ["main", "release"]);
            }
            _ => panic!("expected pull command")
        }
    }

    #[test]
    fn parses_working_dir_with_pull_arguments()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "-C",
            tmp.path().to_str().unwrap(),
            "pull",
            "origin",
            "main"
        ]);

        // Assert
        assert_eq!(cli.working_dir.as_deref(), Some(tmp.path()));
        match cli.command
        {
            Command::Pull { repository, refspecs } =>
            {
                assert_eq!(repository.as_deref(), Some("origin"));
                assert_eq!(refspecs, ["main"]);
            }
            _ => panic!("expected pull command")
        }
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

    #[test]
    fn parses_rebase_with_upstream()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "rebase", "origin/main"]);

        // Assert
        match cli.command
        {
            Command::Rebase { upstream } =>
            {
                assert_eq!(upstream, "origin/main");
            }
            _ => panic!("expected rebase command")
        }
    }

    #[test]
    fn rejects_rebase_without_upstream()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "rebase"])
        {
            Ok(_) => panic!("expected rebase parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn parses_switch_with_branch_name()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "switch", "feature/auth"]);

        // Assert
        match cli.command
        {
            Command::Switch { branch_name } =>
            {
                assert_eq!(branch_name, "feature/auth");
            }
            _ => panic!("expected switch command")
        }
    }

    #[test]
    fn rejects_switch_without_branch_name()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "switch"])
        {
            Ok(_) => panic!("expected switch parse to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }
}
