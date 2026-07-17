//! Run pipeline: the fixed sequence every invocation passes through after
//! parsing — build context, report load warnings, execute and report the
//! command, record analytics, check for updates, save state. Notices never
//! fail the command; the exit code mirrors the command result.

use super::Cli;
use super::context::CliContext;
use super::event_meta::CliMetadata;
use crate::analytics::{self, Analytics, AnalyticsState, CommandEvent};
use crate::commands::Command;
use crate::config::GlobalConfig;
use crate::render;
use crate::state::GlobalState;
use crate::store::FileStore;
use crate::updates::{self, Frequency};
use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

/// Run the pipeline with the production adapters: resolved global config
/// and state paths, the process stderr, the analytics recorder, and the
/// update checker.
pub fn run(cli: Cli) -> Result<()>
{
    run_with(
        &cli.display_name,
        &cli.working_dir,
        GlobalConfig::resolve_path(None)?,
        GlobalState::resolve_path(None)?,
        cli.command,
        cli.metadata,
        &mut anstream::stderr(),
        analytics::record,
        updates::check
    )
}

/// Build the context and run the pipeline in order: report load warnings,
/// execute and report the command, record analytics, check for updates,
/// save state. Notices go to `stderr` and never fail the command; the
/// returned result mirrors the command's.
#[expect(
    clippy::too_many_arguments,
    reason = "the pipeline's test surface names every injected dependency"
)]
fn run_with(
    display_name: &str,
    working_dir: &Option<PathBuf>,
    config_path: PathBuf,
    state_path: PathBuf,
    command: Command,
    metadata: CliMetadata,
    stderr: &mut impl Write,
    recorder: impl FnOnce(&Analytics, &mut AnalyticsState, CommandEvent),
    checker: impl FnOnce(Frequency, &mut FileStore<GlobalState>) -> Option<String>
) -> Result<()>
{
    // Build context from the resolved global config and state paths
    let mut context =
        CliContext::new(display_name, working_dir, config_path, state_path)?;

    // Report context load warnings without failing the command
    for warning in context.warnings()
    {
        let _ = writeln!(stderr, "warning: {warning}");
    }

    // Run command
    let start = Instant::now();
    let result = command.run(&context);
    let result = render::emit(result);

    // Record analytics
    recorder(
        &context.global_config.analytics,
        &mut context.global_state.analytics,
        CommandEvent {
            name: metadata.name,
            success: result.is_ok(),
            duration_ms: start.elapsed().as_millis(),
            flags: metadata.flags,
            global_flags: metadata.global_flags
        }
    );

    // Check for updates
    if let Some(update) = checker(
        context.global_config.updates.check_frequency,
        &mut context.global_state
    )
    {
        let _ = writeln!(stderr, "{update}");
    }

    // Save changed context data
    if let Err(err) = context.save()
    {
        let _ = writeln!(stderr, "warning: {err:#}");
    }

    result
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::commands::WorkspaceCommand;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;

    fn metadata(name: &str) -> CliMetadata
    {
        CliMetadata {
            name: name.to_owned(),
            flags: vec!["all".to_owned()],
            global_flags: vec!["working_dir".to_owned()]
        }
    }

    /// A VMR root with no child repos: status runs across zero repos
    /// without spawning git.
    fn empty_vmr(dir: &Path)
    {
        fs::create_dir(dir.join(".gitvmr")).unwrap();
    }

    /// Run a status command through the pipeline, capturing what reached
    /// the injected stderr, recorder, and checker.
    fn run_status(
        working_dir: &Path,
        state_path: PathBuf,
        stderr: &mut Vec<u8>,
        recorder: impl FnOnce(&Analytics, &mut AnalyticsState, CommandEvent),
        checker: impl FnOnce(
            Frequency,
            &mut FileStore<GlobalState>
        ) -> Option<String>
    ) -> Result<()>
    {
        run_with(
            "git vmr",
            &Some(working_dir.to_path_buf()),
            working_dir.join("config.toml"),
            state_path,
            Command::Workspace(WorkspaceCommand::Status),
            metadata("status"),
            stderr,
            recorder,
            checker
        )
    }

    #[test]
    fn failing_command_records_failure_and_keeps_its_exit()
    {
        // Arrange: no VMR marker, so status fails before any git spawn
        let tmp = tempfile::tempdir().unwrap();
        let event = RefCell::new(None);
        let mut stderr = Vec::new();

        // Act
        let result = run_status(
            tmp.path(),
            tmp.path().join("state.toml"),
            &mut stderr,
            |_, _, e| *event.borrow_mut() = Some(e),
            |_, _| None
        );

        // Assert
        assert!(result.is_err());
        let event = event.into_inner().expect("recorder was not called");
        assert!(!event.success);
        assert_eq!(event.name, "status");
        assert_eq!(event.flags, ["all"]);
        assert_eq!(event.global_flags, ["working_dir"]);
    }

    #[test]
    fn successful_command_records_then_checks_in_order()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        empty_vmr(tmp.path());
        let order = RefCell::new(Vec::new());
        let mut stderr = Vec::new();

        // Act
        let result = run_status(
            tmp.path(),
            tmp.path().join("state.toml"),
            &mut stderr,
            |_, _, event| {
                assert!(event.success);
                order.borrow_mut().push("record");
            },
            |_, _| {
                order.borrow_mut().push("check");
                None
            }
        );

        // Assert
        assert!(result.is_ok());
        assert_eq!(order.into_inner(), ["record", "check"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn update_notice_reaches_stderr()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        empty_vmr(tmp.path());
        let mut stderr = Vec::new();

        // Act
        let result = run_status(
            tmp.path(),
            tmp.path().join("state.toml"),
            &mut stderr,
            |_, _, _| (),
            |_, _| Some("\nA new git-vmr version is available: 9.9.9".into())
        );

        // Assert
        assert!(result.is_ok());
        let stderr = String::from_utf8(stderr).unwrap();
        assert_eq!(stderr, "\nA new git-vmr version is available: 9.9.9\n");
    }

    #[test]
    fn malformed_state_warns_without_failing_the_command()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        empty_vmr(tmp.path());
        let state_path = tmp.path().join("state.toml");
        fs::write(&state_path, "not = valid = toml").unwrap();
        let mut stderr = Vec::new();

        // Act
        let result = run_status(
            tmp.path(),
            state_path,
            &mut stderr,
            |_, _, _| (),
            |_, _| None
        );

        // Assert
        assert!(result.is_ok());
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(stderr.starts_with("warning: "));
        assert!(stderr.contains("using defaults"));
    }

    #[test]
    fn save_failure_warns_without_changing_the_exit_code()
    {
        // Arrange: a directory at the state path is unreadable (poisoning
        // the store so save always writes) and unwritable (failing it)
        let tmp = tempfile::tempdir().unwrap();
        empty_vmr(tmp.path());
        let state_path = tmp.path().join("state.toml");
        fs::create_dir(&state_path).unwrap();
        let mut stderr = Vec::new();

        // Act
        let result = run_status(
            tmp.path(),
            state_path,
            &mut stderr,
            |_, _, _| (),
            |_, _| None
        );

        // Assert
        assert!(result.is_ok());
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(stderr.contains("using defaults"));
        assert!(stderr.contains("warning: failed to"));
    }
}
