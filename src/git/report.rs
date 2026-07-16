//! Report policies: the declarative per-operation rule for how a repo
//! outcome's message is produced — which output streams are read in which
//! order, whether an empty result means quiet success or a canned fallback,
//! and any transform applied to the message before it reports.

use crate::git::{
    GitCommandResult, GitOutput, failure_message, quiet_success, stderr,
    success_message
};
use crate::vmr::Repo;
use std::path::Path;

/// The output streams a report policy reads, in priority order.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Streams
{
    StdoutThenStderr,
    StderrThenStdout,
    StdoutOnly,
    StderrOnly
}

impl Streams
{
    /// The first non-empty line read in this priority order, or `fallback`
    /// when every stream read is blank.
    pub(crate) fn first_line(self, output: &GitOutput, fallback: &str)
    -> String
    {
        let (primary, secondary) = match self
        {
            Self::StdoutThenStderr => (&output.stdout, Some(&output.stderr)),
            Self::StderrThenStdout => (&output.stderr, Some(&output.stdout)),
            Self::StdoutOnly => (&output.stdout, None),
            Self::StderrOnly => (&output.stderr, None)
        };

        first_line(primary)
            .or_else(|| secondary.and_then(|bytes| first_line(bytes)))
            .unwrap_or_else(|| fallback.to_owned())
    }
}

/// How a successful operation reports: where the message comes from, and an
/// optional transform applied to every message the policy emits.
pub(crate) struct SuccessReport
{
    source: SuccessSource,
    transform: Option<fn(String) -> String>
}

enum SuccessSource
{
    /// Never reports.
    Quiet,
    /// Always reports this exact message.
    Fixed(String),
    /// Reports the first non-empty line from `from`.
    Line
    {
        from: Streams, on_empty: OnEmpty
    }
}

impl SuccessReport
{
    /// Never reports.
    pub(crate) fn quiet() -> Self
    {
        Self::with_source(SuccessSource::Quiet)
    }

    /// Always reports this exact message.
    pub(crate) fn fixed(message: String) -> Self
    {
        Self::with_source(SuccessSource::Fixed(message))
    }

    /// Reports the first non-empty line from `from`.
    pub(crate) fn line(from: Streams, on_empty: OnEmpty) -> Self
    {
        Self::with_source(SuccessSource::Line { from, on_empty })
    }

    /// Applies `transform` to every message this policy emits.
    pub(crate) fn map(self, transform: fn(String) -> String) -> Self
    {
        Self { transform: Some(transform), ..self }
    }

    fn with_source(source: SuccessSource) -> Self
    {
        Self { source, transform: None }
    }

    fn message(&self, output: &GitOutput) -> Option<String>
    {
        let message = match &self.source
        {
            SuccessSource::Quiet => None,
            SuccessSource::Fixed(message) => Some(message.clone()),
            SuccessSource::Line { from, on_empty } =>
            {
                let line = from.first_line(output, "");

                if line.is_empty()
                {
                    match on_empty
                    {
                        OnEmpty::Quiet => None,
                        OnEmpty::Text(text) => Some((*text).to_owned())
                    }
                }
                else
                {
                    Some(line)
                }
            }
        };

        match self.transform
        {
            Some(transform) => message.map(transform),
            None => message
        }
    }
}

/// What a [`SuccessReport::line`] reports when its streams are blank.
pub(crate) enum OnEmpty
{
    Quiet,
    Text(&'static str)
}

/// How a failed operation reports: where the message comes from, and an
/// optional transform applied to every message the policy emits.
pub(crate) struct FailureReport
{
    source: FailureSource,
    transform: Option<fn(String) -> String>
}

enum FailureSource
{
    /// The first non-empty line from `from`, or `fallback`.
    Line
    {
        from: Streams, fallback: &'static str
    },
    /// `git {command} failed for '{path}': {stderr}`.
    Detailed
    {
        command: &'static str
    },
    /// The last non-empty stderr line — git prints its summary error last —
    /// else the first non-empty stdout line, else `fallback`.
    LastStderrLine
    {
        fallback: &'static str
    }
}

impl FailureReport
{
    /// The first non-empty line from `from`, or `fallback`.
    pub(crate) fn line(from: Streams, fallback: &'static str) -> Self
    {
        Self::with_source(FailureSource::Line { from, fallback })
    }

    /// `git {command} failed for '{path}': {stderr}`.
    pub(crate) fn detailed(command: &'static str) -> Self
    {
        Self::with_source(FailureSource::Detailed { command })
    }

    /// The last non-empty stderr line — git prints its summary error last —
    /// else the first non-empty stdout line, else `fallback`.
    pub(crate) fn last_stderr_line(fallback: &'static str) -> Self
    {
        Self::with_source(FailureSource::LastStderrLine { fallback })
    }

    /// Applies `transform` to every message this policy emits.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "mirrors SuccessReport::map; no operation transforms \
                      a failure message yet"
        )
    )]
    pub(crate) fn map(self, transform: fn(String) -> String) -> Self
    {
        Self { transform: Some(transform), ..self }
    }

    fn with_source(source: FailureSource) -> Self
    {
        Self { source, transform: None }
    }

    fn message(&self, repo_path: &Path, output: &GitOutput) -> String
    {
        let message = match &self.source
        {
            FailureSource::Line { from, fallback } =>
                from.first_line(output, fallback),
            FailureSource::Detailed { command } => format!(
                "git {command} failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            ),
            FailureSource::LastStderrLine { fallback } =>
                last_line(&output.stderr).unwrap_or_else(|| {
                    Streams::StdoutOnly.first_line(output, fallback)
                }),
        };

        match self.transform
        {
            Some(transform) => transform(message),
            None => message
        }
    }
}

/// Maps a git invocation's output to a repo outcome by applying the
/// operation's report policies.
pub(crate) fn command_result(
    repo: &Repo,
    output: &GitOutput,
    success: SuccessReport,
    failure: FailureReport
) -> GitCommandResult
{
    if output.status.success()
    {
        Ok(match success.message(output)
        {
            None => quiet_success(),
            Some(message) => success_message(&repo.name, message)
        })
    }
    else
    {
        Ok(failure_message(&repo.name, failure.message(&repo.path, output)))
    }
}

fn first_line(bytes: &[u8]) -> Option<String>
{
    String::from_utf8_lossy(bytes)
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::to_owned)
}

fn last_line(bytes: &[u8]) -> Option<String>
{
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::exit_status;
    use crate::test_support::repo;

    fn output(code: i32, stdout: &str, stderr: &str) -> GitOutput
    {
        GitOutput {
            status: exit_status(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec()
        }
    }

    fn backend() -> Repo
    {
        repo("backend", "/vmr/backend")
    }

    fn success_text(outcome: RepoOutcome) -> Option<String>
    {
        match outcome
        {
            RepoOutcome::Success(message) =>
                message.map(|message| message.message),
            RepoOutcome::Failure(_) => panic!("expected success")
        }
    }

    fn failure_text(outcome: RepoOutcome) -> String
    {
        match outcome
        {
            RepoOutcome::Failure(message) => message.message,
            RepoOutcome::Success(_) => panic!("expected failure")
        }
    }

    fn shout(message: String) -> String
    {
        message.to_uppercase()
    }

    #[test]
    fn streams_read_in_declared_priority_order()
    {
        let both = output(0, "out line\n", "err line\n");

        for (streams, expected) in [
            (Streams::StdoutThenStderr, "out line"),
            (Streams::StderrThenStdout, "err line"),
            (Streams::StdoutOnly, "out line"),
            (Streams::StderrOnly, "err line")
        ]
        {
            assert_eq!(streams.first_line(&both, "fallback"), expected);
        }
    }

    #[test]
    fn two_stream_orders_fall_through_to_the_secondary_stream()
    {
        let stderr_only = output(0, "  \n", "err line\n");
        let stdout_only = output(0, "out line\n", "");

        assert_eq!(
            Streams::StdoutThenStderr.first_line(&stderr_only, "fallback"),
            "err line"
        );
        assert_eq!(
            Streams::StderrThenStdout.first_line(&stdout_only, "fallback"),
            "out line"
        );
    }

    #[test]
    fn single_stream_orders_ignore_the_other_stream()
    {
        let both = output(0, "out line\n", "err line\n");
        let blank = output(0, "", "err line\n");

        assert_eq!(Streams::StdoutOnly.first_line(&both, "fb"), "out line");
        assert_eq!(Streams::StdoutOnly.first_line(&blank, "fb"), "fb");
    }

    #[test]
    fn quiet_success_reports_nothing()
    {
        let outcome = command_result(
            &backend(),
            &output(0, "chatter\n", "chatter\n"),
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrThenStdout, "failed")
        )
        .unwrap();

        assert_eq!(success_text(outcome), None);
    }

    #[test]
    fn fixed_success_reports_the_exact_message()
    {
        let outcome = command_result(
            &backend(),
            &output(0, "chatter\n", ""),
            SuccessReport::fixed("Deleted branch topic".to_owned()),
            FailureReport::line(Streams::StderrThenStdout, "failed")
        )
        .unwrap();

        assert_eq!(
            success_text(outcome),
            Some("Deleted branch topic".to_owned())
        );
    }

    #[test]
    fn line_success_with_empty_streams_honors_on_empty()
    {
        for (on_empty, expected) in [
            (OnEmpty::Quiet, None),
            (OnEmpty::Text("git op succeeded"), Some("git op succeeded"))
        ]
        {
            let outcome = command_result(
                &backend(),
                &output(0, "", ""),
                SuccessReport::line(Streams::StdoutThenStderr, on_empty),
                FailureReport::line(Streams::StderrThenStdout, "failed")
            )
            .unwrap();

            assert_eq!(success_text(outcome), expected.map(str::to_owned));
        }
    }

    #[test]
    fn line_success_reports_the_first_non_empty_line()
    {
        let outcome = command_result(
            &backend(),
            &output(0, "\nfirst real line\nsecond line\n", ""),
            SuccessReport::line(Streams::StdoutThenStderr, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "failed")
        )
        .unwrap();

        assert_eq!(success_text(outcome), Some("first real line".to_owned()));
    }

    #[test]
    fn line_failure_reports_line_or_fallback()
    {
        for (stderr, expected) in
            [("error: denied\n", "error: denied"), ("", "git push failed")]
        {
            let outcome = command_result(
                &backend(),
                &output(1, "", stderr),
                SuccessReport::quiet(),
                FailureReport::line(
                    Streams::StderrThenStdout,
                    "git push failed"
                )
            )
            .unwrap();

            assert_eq!(failure_text(outcome), expected);
        }
    }

    #[test]
    fn detailed_failure_formats_command_path_and_stderr()
    {
        let outcome = command_result(
            &backend(),
            &output(1, "", "pathspec did not match\n"),
            SuccessReport::quiet(),
            FailureReport::detailed("add")
        )
        .unwrap();

        assert_eq!(
            failure_text(outcome),
            "git add failed for '/vmr/backend': pathspec did not match"
        );
    }

    #[test]
    fn last_stderr_line_failure_prefers_the_final_stderr_line()
    {
        for ((stdout, stderr), expected) in [
            (
                ("", "Preparing worktree\nfatal: already exists\n"),
                "fatal: already exists"
            ),
            (("stdout detail\n", ""), "stdout detail"),
            (("", ""), "git worktree add failed")
        ]
        {
            let outcome = command_result(
                &backend(),
                &output(1, stdout, stderr),
                SuccessReport::quiet(),
                FailureReport::last_stderr_line("git worktree add failed")
            )
            .unwrap();

            assert_eq!(failure_text(outcome), expected);
        }
    }

    #[test]
    fn success_transform_applies_to_every_message_the_policy_emits()
    {
        for (success, expected) in [
            (
                SuccessReport::line(Streams::StdoutOnly, OnEmpty::Quiet),
                Some("STREAM LINE")
            ),
            (
                SuccessReport::fixed("fixed message".to_owned()),
                Some("FIXED MESSAGE")
            ),
            (
                SuccessReport::line(
                    Streams::StderrOnly,
                    OnEmpty::Text("fallback text")
                ),
                Some("FALLBACK TEXT")
            ),
            (SuccessReport::quiet(), None)
        ]
        {
            let outcome = command_result(
                &backend(),
                &output(0, "stream line\n", ""),
                success.map(shout),
                FailureReport::line(Streams::StderrThenStdout, "failed")
            )
            .unwrap();

            assert_eq!(success_text(outcome), expected.map(str::to_owned));
        }
    }

    #[test]
    fn failure_transform_applies_to_every_message_the_policy_emits()
    {
        for (failure, expected) in [
            (
                FailureReport::line(Streams::StderrThenStdout, "failed"),
                "ERROR: DENIED"
            ),
            (
                FailureReport::line(Streams::StdoutOnly, "canned fallback"),
                "CANNED FALLBACK"
            ),
            (
                FailureReport::detailed("add"),
                "GIT ADD FAILED FOR '/VMR/BACKEND': ERROR: DENIED"
            ),
            (FailureReport::last_stderr_line("failed"), "ERROR: DENIED")
        ]
        {
            let outcome = command_result(
                &backend(),
                &output(1, "", "error: denied\n"),
                SuccessReport::quiet(),
                failure.map(shout)
            )
            .unwrap();

            assert_eq!(failure_text(outcome), expected);
        }
    }
}
