mod context;
mod error;
mod event_meta;

use crate::analytics::CommandEvent;
use crate::commands::Command;
use crate::{analytics, render, updates};
use anyhow::Result;
use clap::{ArgAction, CommandFactory, Error, FromArgMatches, Parser};
pub use context::CliContext;
pub use error::SilentError;
use event_meta::{CommandEventMeta, event_meta};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

pub const APP_NAME: &str = "git-vmr";

#[derive(Parser)]
#[command(name = APP_NAME, version, disable_version_flag = true)]
pub struct Cli
{
    /// Display name of the application
    #[arg(skip)]
    display_name: String,

    /// Event metadata derived from the parsed command line
    #[arg(skip)]
    event_meta: CommandEventMeta,

    /// Run as if git-vmr was started in <path> instead of the current working
    /// directory
    #[arg(short = 'C', value_name = "path")]
    working_dir: Option<PathBuf>,

    /// Print version
    #[arg(short, long, action = ArgAction::Version)]
    version: (),

    /// The command to execute
    #[command(subcommand)]
    command: Command
}

impl Cli
{
    /// Parse arguments
    pub fn parse() -> Self
    {
        Self::parse_from(env::args_os()).unwrap_or_else(|err| err.exit())
    }

    /// Parse arguments from iterator
    fn parse_from<I, T>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone
    {
        // Parse arguments
        let mut command = Self::command();
        let mut matches = command.try_get_matches_from_mut(args)?;
        let meta = event_meta(&command, &matches);
        let mut cli = Self::from_arg_matches_mut(&mut matches)?;
        cli.event_meta = meta;

        // Set display name
        cli.display_name =
            match command.get_bin_name().unwrap_or_else(|| command.get_name())
            {
                "git-vmr" => "git vmr".to_owned(),
                binary_name => binary_name.to_owned()
            };

        Ok(cli)
    }

    /// Run command
    pub fn run(self) -> Result<()>
    {
        // Build context
        let mut context =
            CliContext::new(&self.display_name, &self.working_dir)?;

        // Report context load warnings without failing the command
        for warning in &context.warnings
        {
            eprintln!("warning: {warning}");
        }

        let result = if context.global_config.analytics.enabled()
        {
            // Prepare analytics event
            let start = Instant::now();
            let meta = self.event_meta;

            // Run command
            let result = self.command.run(&context);

            // Record analytics event
            analytics::record(&mut context, CommandEvent {
                name: meta.name,
                success: result.is_ok(),
                duration_ms: start.elapsed().as_millis(),
                flags: meta.flags,
                global_flags: meta.global_flags
            });

            result
        }
        else
        {
            // Run command
            self.command.run(&context)
        };

        // Print rendered command output at the single choke point
        let result = render::emit(result);

        // Check for updates
        if let Some(update) = updates::check(&mut context)
        {
            eprintln!("{update}");
        }

        // Save changed context data
        if let Err(err) = context.save()
        {
            eprintln!("warning: {err:#}");
        }

        result
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn captures_invoked_command_name()
    {
        // Act
        let cli = Cli::parse_from(["vv", "status"]).unwrap();

        // Assert
        assert_eq!(cli.display_name, "vv");
    }

    #[test]
    fn maps_canonical_binary_name_to_git_subcommand()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "status"]).unwrap();

        // Assert
        assert_eq!(cli.display_name, "git vmr");
    }

    #[test]
    fn captures_invoked_command_file_name()
    {
        // Act
        let cli = Cli::parse_from(["/usr/local/bin/vv", "status"]).unwrap();

        // Assert
        assert_eq!(cli.display_name, "vv");
    }

    #[test]
    fn parses_working_dir_argument_before_subcommand()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let cli = Cli::parse_from([
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
        let cli = Cli::parse_from([
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
    fn captures_analytics_metadata_for_value_flags_without_values()
    {
        let cli = Cli::parse_from([
            "git-vmr",
            "-C",
            "/tmp",
            "add",
            "--chmod=+x",
            "backend/src.rs"
        ])
        .unwrap();

        assert_eq!(cli.event_meta.global_flags, ["working_dir"]);
        assert_eq!(cli.event_meta.name, "add");
        assert_eq!(cli.event_meta.flags, ["chmod"]);
    }

    #[test]
    fn captures_analytics_metadata_for_worktree_subcommands()
    {
        let cli = Cli::parse_from([
            "git-vmr",
            "worktree",
            "remove",
            "--force",
            "--delete",
            "../feature"
        ])
        .unwrap();

        assert_eq!(cli.event_meta.name, "worktree.remove");
        assert_eq!(cli.event_meta.flags, ["force", "delete"]);
    }

    #[test]
    fn captures_analytics_metadata_for_foreach_without_command_value()
    {
        let cli = Cli::parse_from([
            "git-vmr", "foreach", "--quiet", "echo", "secret"
        ])
        .unwrap();

        assert_eq!(cli.event_meta.name, "foreach");
        assert_eq!(cli.event_meta.flags, ["quiet"]);
    }

    #[test]
    fn parses_add_long_flags()
    {
        // Act
        let cli = Cli::parse_from([
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
        let cli = Cli::parse_from(["git-vmr", "add", "-A"]).unwrap();

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
        let err = match Cli::parse_from(["git-vmr", "add"])
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
        let err = match Cli::parse_from([
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
    fn parses_rm_short_flags()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "rm",
            "-r",
            "-f",
            "-n",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Rm {
                recursive: true,
                force: true,
                dry_run: true,
                cached: false,
                paths
            } if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_rm_long_flags()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "rm",
            "--force",
            "--dry-run",
            "--cached",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Rm {
                recursive: false,
                force: true,
                dry_run: true,
                cached: true,
                paths
            } if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_mv_two_or_more_paths()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "mv", "one", "two", "three"]).unwrap();

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
            let err = Cli::parse_from(args).err().unwrap();

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
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "add", "../wt"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Add {
                    branch: None,
                    path,
                    commit_ish: None
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_add_optional_commit_ish()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "add", "../wt", "main"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Add {
                    branch: None,
                    path,
                    commit_ish: Some(commit_ish)
                })
            } if &path == "../wt" && commit_ish == "main"
        ));
    }

    #[test]
    fn parses_worktree_add_branch()
    {
        // Act
        let cli = Cli::parse_from([
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
                command: Some(crate::commands::WorktreeCommand::Add {
                    branch: Some(branch),
                    path,
                    commit_ish: None
                })
            } if branch == "feature/auth" && &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_add_branch_with_commit_ish()
    {
        // Act
        let cli = Cli::parse_from([
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
                command: Some(crate::commands::WorktreeCommand::Add {
                    branch: Some(branch),
                    path,
                    commit_ish: Some(commit_ish)
                })
            } if branch == "feature/auth" && &path == "../wt" && commit_ish == "main"
        ));
    }

    #[test]
    fn rejects_worktree_add_branch_without_value()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "worktree", "add", "-b"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
    }

    #[test]
    fn rejects_worktree_add_long_branch_alias()
    {
        // Act
        let err = Cli::parse_from([
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
            Cli::parse_from(["git-vmr", "worktree", "add"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_foreach_without_command()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "foreach"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn parses_foreach_quiet_mode()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "foreach", "--quiet", "echo", "ok"])
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
        let cli =
            Cli::parse_from(["git-vmr", "foreach", "git", "status", "--short"])
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
        let cli = Cli::parse_from([
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
        let err = Cli::parse_from([
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
        let cli = Cli::parse_from(["git-vmr", "worktree", "list"]).unwrap();

        // Assert
        assert!(matches!(cli.command, Command::Worktree {
            command: Some(crate::commands::WorktreeCommand::List)
        }));
    }

    #[test]
    fn parses_worktree_without_subcommand_as_default_list()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "worktree"]).unwrap();

        // Assert
        assert!(matches!(cli.command, Command::Worktree { command: None }));
    }

    #[test]
    fn rejects_worktree_list_extra_operands()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "worktree", "list", "extra"])
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
            let err = Cli::parse_from(["git-vmr", "worktree", "list", option])
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
        let cli = Cli::parse_from(["git-vmr", "worktree", "remove", "../wt"])
            .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 0,
                    delete: false,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_rm_alias_target_path()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "rm", "../wt"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 0,
                    delete: false,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_single_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--force", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 1,
                    delete: false,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_repeated_long_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--force", "--force", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 2,
                    delete: false,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_repeated_short_force()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "remove", "-ff", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 2,
                    delete: false,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_short_delete()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "remove", "-d", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 0,
                    delete: true,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_long_delete()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--delete", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 0,
                    delete: true,
                    force_delete: false,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_force_delete()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "worktree", "remove", "-D", "../wt"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 0,
                    delete: false,
                    force_delete: true,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_force_and_force_delete()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "remove", "--force", "-D", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 1,
                    delete: false,
                    force_delete: true,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn parses_worktree_remove_rm_alias_force_and_force_delete()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "rm", "--force", "-D", "../wt"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Remove {
                    force: 1,
                    delete: false,
                    force_delete: true,
                    path
                })
            } if &path == "../wt"
        ));
    }

    #[test]
    fn rejects_worktree_remove_conflicting_delete_modes()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr", "worktree", "remove", "-d", "-D", "../wt"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn rejects_worktree_remove_long_force_delete()
    {
        // Act
        let err = Cli::parse_from([
            "git-vmr",
            "worktree",
            "remove",
            "--force-delete",
            "../wt"
        ])
        .err()
        .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_worktree_remove_without_target_path()
    {
        // Act
        let err =
            Cli::parse_from(["git-vmr", "worktree", "remove"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_remove_extra_operands()
    {
        // Act
        let err = Cli::parse_from([
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
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Move {
                    force: 0,
                    path,
                    new_path
                })
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_single_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "--force", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Move {
                    force: 1,
                    path,
                    new_path
                })
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_repeated_long_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "--force", "--force", "../wt",
            "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Move {
                    force: 2,
                    path,
                    new_path
                })
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn parses_worktree_move_repeated_short_force()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr", "worktree", "move", "-ff", "../wt", "../moved"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Worktree {
                command: Some(crate::commands::WorktreeCommand::Move {
                    force: 2,
                    path,
                    new_path
                })
            } if &path == "../wt" && &new_path == "../moved"
        ));
    }

    #[test]
    fn rejects_worktree_move_without_source_path()
    {
        // Act
        let err =
            Cli::parse_from(["git-vmr", "worktree", "move"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_move_without_destination_path()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "worktree", "move", "../wt"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_worktree_move_extra_operands()
    {
        // Act
        let err = Cli::parse_from([
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
        Cli::parse_from(["git-vmr", "worktree", "list"]).unwrap();
        let err = Cli::parse_from(["git-vmr", "worktree", "lock", "../wt"])
            .err()
            .unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn long_version_flag_displays_version()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "--version"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn short_version_flag_displays_version()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "-v"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn default_short_version_flag_is_disabled()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "-V"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }
}
