use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;

/// Set `HEAD` or the index to a known state
#[derive(clap::Args)]
pub struct ResetArgs
{
    /// Leave your working directory unchanged
    #[arg(long, conflicts_with_all = ["soft", "hard", "merge", "keep"])]
    pub mixed: bool,

    /// Leave your working tree files and the index unchanged
    #[arg(long, conflicts_with_all = ["mixed", "hard", "merge", "keep"])]
    pub soft: bool,

    /// Overwrite all files and directories with the version from [commit],
    /// and may overwrite untracked files
    #[arg(long, conflicts_with_all = ["soft", "mixed", "merge", "keep"])]
    pub hard: bool,

    /// Reset the index and update the files in the working tree that are
    /// different between [commit] and HEAD, but keep those which are
    /// different between the index and working tree (i.e. which have
    /// changes which have not been added)
    #[arg(long, conflicts_with_all = ["soft", "mixed", "hard", "keep"])]
    pub merge: bool,

    /// Resets index entries and updates files in the working tree that are
    /// different between [commit] and HEAD
    #[arg(long, conflicts_with_all = ["soft", "mixed", "hard", "merge"])]
    pub keep: bool,

    /// Set the current branch head (HEAD) to point at [commit]
    #[arg(value_name = "commit")]
    pub commit: Option<String>
}

impl ResetArgs
{
    /// The selected reset mode. Total: the `conflicts_with_all` attributes
    /// above reject every multi-flag combination.
    pub fn mode(&self) -> Option<ResetMode>
    {
        match (self.soft, self.mixed, self.hard, self.merge, self.keep)
        {
            (true, false, false, false, false) => Some(ResetMode::Soft),
            (false, true, false, false, false) => Some(ResetMode::Mixed),
            (false, false, true, false, false) => Some(ResetMode::Hard),
            (false, false, false, true, false) => Some(ResetMode::Merge),
            (false, false, false, false, true) => Some(ResetMode::Keep),
            (false, false, false, false, false) => None,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetMode
{
    Soft,
    Mixed,
    Hard,
    Merge,
    Keep
}

impl ResetMode
{
    pub fn as_arg(self) -> &'static str
    {
        match self
        {
            ResetMode::Soft => "--soft",
            ResetMode::Mixed => "--mixed",
            ResetMode::Hard => "--hard",
            ResetMode::Merge => "--merge",
            ResetMode::Keep => "--keep"
        }
    }
}

impl Git
{
    pub fn reset(
        &self,
        repo: &Repo,
        mode: Option<ResetMode>,
        commit: Option<&str>
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("reset")];

        if let Some(mode) = mode
        {
            args.push(OsString::from(mode.as_arg()));
        }

        if let Some(commit) = commit
        {
            args.push(OsString::from(commit));
        }

        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::line(Streams::StdoutThenStderr, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "git reset failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;
    use crate::test_support::repo;

    fn args(
        soft: bool,
        mixed: bool,
        hard: bool,
        merge: bool,
        keep: bool
    ) -> ResetArgs
    {
        ResetArgs { soft, mixed, hard, merge, keep, commit: None }
    }

    #[test]
    fn mode_returns_none_when_no_mode_flag_is_set()
    {
        // Act
        let mode = args(false, false, false, false, false).mode();

        // Assert
        assert_eq!(mode, None);
    }

    #[test]
    fn mode_returns_selected_mode()
    {
        for (flags, expected) in [
            ((true, false, false, false, false), ResetMode::Soft),
            ((false, true, false, false, false), ResetMode::Mixed),
            ((false, false, true, false, false), ResetMode::Hard),
            ((false, false, false, true, false), ResetMode::Merge),
            ((false, false, false, false, true), ResetMode::Keep)
        ]
        {
            // Act
            let mode = args(flags.0, flags.1, flags.2, flags.3, flags.4).mode();

            // Assert
            assert_eq!(mode, Some(expected));
        }
    }

    #[test]
    fn reset_success_reports_stdout_before_stderr()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["reset"],
            0,
            "Unstaged changes after reset:\n",
            "stderr chatter\n"
        ));

        // Act
        let outcome =
            git.reset(&repo("backend", "/vmr/backend"), None, None).unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "Unstaged changes after reset:");
    }
}
