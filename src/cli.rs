use crate::commands::Command;
use anyhow::{Context, Result, bail};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

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

        // Run command in working directory
        self.command.run(&self.bin_name, &working_dir)
    }

    fn get_working_dir(&self) -> Result<PathBuf>
    {
        // Parse working directory from argument
        if let Some(working_dir) = &self.working_dir
        {
            // Get absolute path
            let absolute_path =
                working_dir.canonicalize().with_context(|| {
                    format!(
                        "fatal: cannot change to '{}'",
                        working_dir.display()
                    )
                })?;

            // Ensure the path is a directory
            if !absolute_path.is_dir()
            {
                bail!(
                    "fatal: cannot change to '{}': Not a directory",
                    working_dir.display()
                );
            }

            return Ok(absolute_path);
        }

        // If none provided, use current working directory
        env::current_dir().context("fatal: failed to get current directory")
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
    use clap::error::ErrorKind;
    use std::fs;

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
    fn resolves_working_dir_argument_to_canonical_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let cli = Cli {
            bin_name: "git vmr".to_owned(),
            working_dir: Some(nested.join("..").join("nested")),
            version: (),
            command: Command::Status
        };

        // Act
        let working_dir = cli.get_working_dir().unwrap();

        // Assert
        assert_eq!(working_dir, nested.canonicalize().unwrap());
    }

    #[test]
    fn rejects_working_dir_argument_that_is_not_a_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("file");
        fs::write(&file, "").unwrap();
        let cli = Cli {
            bin_name: "git vmr".to_owned(),
            working_dir: Some(file.clone()),
            version: (),
            command: Command::Status
        };

        // Act
        let err = cli.get_working_dir().unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            format!(
                "fatal: cannot change to '{}': Not a directory",
                file.display()
            )
        );
    }

    #[test]
    fn reports_missing_working_dir_argument_path()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("missing");
        let cli = Cli {
            bin_name: "git vmr".to_owned(),
            working_dir: Some(missing.clone()),
            version: (),
            command: Command::Status
        };

        // Act
        let err = cli.get_working_dir().unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            format!("fatal: cannot change to '{}'", missing.display())
        );
    }

    #[test]
    fn uses_current_dir_when_working_dir_argument_is_missing()
    {
        // Arrange
        let cli = Cli {
            bin_name: "git vmr".to_owned(),
            working_dir: None,
            version: (),
            command: Command::Status
        };

        // Act
        let working_dir = cli.get_working_dir().unwrap();

        // Assert
        assert_eq!(working_dir, env::current_dir().unwrap());
    }

    #[test]
    fn parses_working_dir_argument_before_subcommand()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::try_parse_from([
            "git-vmr",
            "-C",
            tmp.path().to_str().unwrap(),
            "status"
        ])
        .unwrap();

        // Assert
        assert_eq!(cli.working_dir, Some(tmp.path().to_path_buf()));
        assert!(matches!(cli.command, Command::Status));
    }

    #[test]
    fn parses_add_flags()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr",
            "add",
            "-A",
            "-f",
            "--chmod=+x",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Add {
                all: true,
                force: true,
                chmod: Some(crate::git::ChmodMode::Executable),
                paths
            } if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_add_long_flags()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr",
            "add",
            "--all",
            "--force",
            "--chmod=-x",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Add {
                all: true,
                force: true,
                chmod: Some(crate::git::ChmodMode::NotExecutable),
                paths
            } if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_add_all_without_pathspecs()
    {
        // Act
        let cli = Cli::try_parse_from(["git-vmr", "add", "-A"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Add {
                all: true,
                force: false,
                chmod: None,
                paths
            } if paths.is_empty()
        ));
    }

    #[test]
    fn rejects_add_without_pathspecs_or_all()
    {
        // Act
        let err = match Cli::try_parse_from(["git-vmr", "add"])
        {
            Ok(_) => panic!("expected add without pathspecs or all to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_add_invalid_chmod()
    {
        // Act
        let err = match Cli::try_parse_from([
            "git-vmr",
            "add",
            "--chmod=bad",
            "backend/src.rs"
        ])
        {
            Ok(_) => panic!("expected invalid chmod to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn parses_mv_two_or_more_paths()
    {
        // Act
        let cli = Cli::try_parse_from(["git-vmr", "mv", "one", "two", "three"])
            .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Mv {
                sources,
                destination
            } if sources == [PathBuf::from("one"), PathBuf::from("two")]
                && destination == *"three"
        ));
    }

    #[test]
    fn rejects_mv_without_two_paths_or_with_unsupported_options()
    {
        for args in [
            vec!["git-vmr", "mv"],
            vec!["git-vmr", "mv", "one"],
            vec!["git-vmr", "mv", "-r", "one", "two"],
            vec!["git-vmr", "mv", "--dry-run", "one", "two"],
            vec!["git-vmr", "mv", "-k", "one", "two"],
            vec!["git-vmr", "mv", "--force", "one", "two"]
        ]
        {
            // Act
            let err = Cli::try_parse_from(args).err().unwrap();

            // Assert
            assert!(
                matches!(
                    err.kind(),
                    ErrorKind::MissingRequiredArgument
                        | ErrorKind::UnknownArgument
                        | ErrorKind::InvalidValue
                        | ErrorKind::TooFewValues
                ),
                "unexpected error kind: {:?}",
                err.kind()
            );
        }
    }

    #[test]
    fn parses_worktree_add_target_path()
    {
        // Act
        let cli = Cli::try_parse_from(["git-vmr", "worktree", "add", "../wt"])
            .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Add {
                    branch: None,
                    path,
                    commit_ish: None
                }
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_add_optional_commit_ish()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "add", "../wt", "main"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Add {
                    branch: None,
                    path,
                    commit_ish: Some(commit_ish)
                }
            } if &path == "../wt" && commit_ish == "main"
        ));
    }

    #[test]
    fn parses_worktree_add_branch()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr",
            "worktree",
            "add",
            "-b",
            "feature/auth",
            "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Add {
                    branch: Some(branch),
                    path,
                    commit_ish: None
                }
            } if branch == "feature/auth" && &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_add_branch_with_commit_ish()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr",
            "worktree",
            "add",
            "-b",
            "feature/auth",
            "../wt",
            "main"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Add {
                    branch: Some(branch),
                    path,
                    commit_ish: Some(commit_ish)
                }
            } if branch == "feature/auth" && &path == "../wt" && commit_ish == "main"
        ));
    }

    #[test]
    fn rejects_worktree_add_branch_without_value()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "worktree", "add", "-b"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    #[test]
    fn rejects_worktree_add_long_branch_alias()
    {
        // Act
        let err = Cli::try_parse_from([
            "git-vmr",
            "worktree",
            "add",
            "--branch",
            "feature/auth",
            "../wt"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_worktree_add_without_target_path()
    {
        // Act
        let err =
            Cli::try_parse_from(["git-vmr", "worktree", "add"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_foreach_without_command()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "foreach"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn parses_foreach_quiet_mode()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "foreach", "--quiet", "echo", "ok"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Foreach {
                quiet: true,
                command
            } if command == ["echo", "ok"]
        ));
    }

    #[test]
    fn parses_foreach_multi_word_command()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "foreach", "git", "status", "--short"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Foreach {
                quiet: false,
                command
            } if command == ["git", "status", "--short"]
        ));
    }

    #[test]
    fn captures_foreach_child_command_options()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr",
            "foreach",
            "echo",
            "--not-a-vmr-option"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Foreach {
                quiet: false,
                command
            } if command == ["echo", "--not-a-vmr-option"]
        ));
    }

    #[test]
    fn rejects_worktree_add_extra_operands()
    {
        // Act
        let err = Cli::try_parse_from([
            "git-vmr", "worktree", "add", "../wt", "main", "extra"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn parses_worktree_list()
    {
        // Act
        let cli = Cli::try_parse_from(["git-vmr", "worktree", "list"]).unwrap();

        // Assert
        assert!(matches!(cli.command, Command::Worktree {
            command: crate::commands::WorktreeCommand::List
        }));
    }

    #[test]
    fn rejects_worktree_list_extra_operands()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "worktree", "list", "extra"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_worktree_list_options()
    {
        for option in ["--porcelain", "-z", "-v"]
        {
            // Act
            let err =
                Cli::try_parse_from(["git-vmr", "worktree", "list", option])
                    .err()
                    .unwrap();

            // Assert
            assert_eq!(err.kind(), ErrorKind::UnknownArgument);
        }
    }

    #[test]
    fn parses_worktree_remove_target_path()
    {
        // Act
        let cli =
            Cli::try_parse_from(["git-vmr", "worktree", "remove", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Remove {
                    force: 0,
                    path
                }
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_single_force()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "remove", "--force", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Remove {
                    force: 1,
                    path
                }
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_repeated_long_force()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "remove", "--force", "--force", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Remove {
                    force: 2,
                    path
                }
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_repeated_short_force()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "remove", "-ff", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Remove {
                    force: 2,
                    path
                }
            } if &path == "../wt"
        ));
    }

    #[test]
    fn rejects_worktree_remove_without_target_path()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "worktree", "remove"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_remove_extra_operands()
    {
        // Act
        let err = Cli::try_parse_from([
            "git-vmr", "worktree", "remove", "../wt", "extra"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn parses_worktree_move_paths()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "move", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Move {
                    force: 0,
                    path,
                    new_path
                }
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_single_force()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "move", "--force", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Move {
                    force: 1,
                    path,
                    new_path
                }
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_repeated_long_force()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "move", "--force", "--force", "../wt",
            "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Move {
                    force: 2,
                    path,
                    new_path
                }
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_repeated_short_force()
    {
        // Act
        let cli = Cli::try_parse_from([
            "git-vmr", "worktree", "move", "-ff", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: crate::commands::WorktreeCommand::Move {
                    force: 2,
                    path,
                    new_path
                }
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn rejects_worktree_move_without_source_path()
    {
        // Act
        let err =
            Cli::try_parse_from(["git-vmr", "worktree", "move"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_move_without_destination_path()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "worktree", "move", "../wt"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_move_extra_operands()
    {
        // Act
        let err = Cli::try_parse_from([
            "git-vmr", "worktree", "move", "../wt", "../moved", "extra"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_unsupported_worktree_subcommands()
    {
        // Act
        Cli::try_parse_from(["git-vmr", "worktree", "list"]).unwrap();
        let err = Cli::try_parse_from(["git-vmr", "worktree", "lock", "../wt"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn long_version_flag_displays_version()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "--version"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn short_version_flag_displays_version()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "-v"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn default_short_version_flag_is_disabled()
    {
        // Act
        let err = Cli::try_parse_from(["git-vmr", "-V"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }
}
