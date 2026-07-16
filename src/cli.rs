mod context;
mod error;
mod event_meta;

use crate::analytics::CommandEvent;
use crate::commands::Command;
use crate::config::GlobalConfig;
use crate::state::GlobalState;
use crate::{analytics, render, updates};
use anyhow::Result;
use clap::{ArgAction, CommandFactory, Error, FromArgMatches, Parser};
pub use context::CliContext;
pub use error::SilentError;
use event_meta::CliMetadata;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

pub const APP_NAME: &str = "git-vmr";

#[derive(Parser)]
#[command(name = APP_NAME, version, disable_version_flag = true)]
pub struct Cli
{
    /// Metadata derived from the command line arguments
    #[arg(skip)]
    metadata: CliMetadata,

    /// Display name of the application
    #[arg(skip)]
    display_name: String,

    /// Run as if git-vmr was started in <path> instead of the current working
    /// directory
    #[arg(short = 'C', value_name = "path")]
    working_dir: Option<PathBuf>,

    /// Print version
    #[arg(short, long, action = ArgAction::Version)]
    version: (),

    /// The command to execute
    #[command(subcommand)]
    pub(crate) command: Command
}

impl Cli
{
    /// Parse command line arguments
    pub fn parse() -> Self
    {
        Self::parse_from(env::args_os()).unwrap_or_else(|err| err.exit())
    }

    /// Parse the given arguments
    pub(crate) fn parse_from<I, T>(args: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone
    {
        // Parse valid command arguments
        let mut command = Self::command();
        let mut matches = command.try_get_matches_from_mut(args)?;

        // Build command metadata
        let metadata = event_meta::event_meta(&command, &matches);

        // Map valid arguments to typed struct
        let mut cli = Self::from_arg_matches_mut(&mut matches)?;

        // Enrich with metadata
        cli.metadata = metadata;
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
        // Build context from the resolved global config and state paths
        let mut context = CliContext::new(
            &self.display_name,
            &self.working_dir,
            GlobalConfig::resolve_path(None)?,
            GlobalState::resolve_path(None)?
        )?;

        // Report context load warnings without failing the command
        for warning in context.warnings()
        {
            eprintln!("warning: {warning}");
        }

        // Run command
        let start = Instant::now();
        let result = self.command.run(&context);
        let result = render::emit(result);

        // Record analytics
        analytics::record(
            &context.global_config.analytics,
            &mut context.global_state.analytics,
            CommandEvent {
                name: self.metadata.name,
                success: result.is_ok(),
                duration_ms: start.elapsed().as_millis(),
                flags: self.metadata.flags,
                global_flags: self.metadata.global_flags
            }
        );

        // Check for updates
        if let Some(update) = updates::check(
            context.global_config.updates.check_frequency,
            &mut context.global_state
        )
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
    use crate::commands::WorkspaceCommand;
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
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Status)
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

        assert_eq!(cli.metadata.global_flags, ["working_dir"]);
        assert_eq!(cli.metadata.name, "add");
        assert_eq!(cli.metadata.flags, ["chmod"]);
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

        assert_eq!(cli.metadata.name, "worktree.remove");
        assert_eq!(cli.metadata.flags, ["force", "delete"]);
    }

    #[test]
    fn captures_analytics_metadata_for_foreach_without_command_value()
    {
        let cli = Cli::parse_from([
            "git-vmr", "foreach", "--quiet", "echo", "secret"
        ])
        .unwrap();

        assert_eq!(cli.metadata.name, "foreach");
        assert_eq!(cli.metadata.flags, ["quiet"]);
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
