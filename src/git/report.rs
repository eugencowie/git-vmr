//! Report policies: the declarative per-operation rule for how a repo
//! outcome's message is produced — which output streams are read in which
//! order, and whether an empty result means quiet success or a canned
//! fallback.

use crate::git::{
    GitCommandResult, GitOutput, failure_message, quiet_success, stderr,
    success_message
};
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

/// How a successful operation reports.
pub(crate) enum SuccessReport
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

/// What a [`SuccessReport::Line`] reports when its streams are blank.
pub(crate) enum OnEmpty
{
    Quiet,
    Text(&'static str)
}

/// How a failed operation reports.
pub(crate) enum FailureReport
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

/// Maps a git invocation's output to a repo outcome by applying the
/// operation's report policies.
pub(crate) fn command_result(
    repo_name: &str,
    repo_path: &Path,
    output: &GitOutput,
    success: SuccessReport,
    failure: FailureReport
) -> GitCommandResult
{
    if output.status.success()
    {
        Ok(match success
        {
            SuccessReport::Quiet => quiet_success(),
            SuccessReport::Fixed(message) =>
                success_message(repo_name, message),
            SuccessReport::Line { from, on_empty } =>
            {
                let line = from.first_line(output, "");

                if line.is_empty()
                {
                    match on_empty
                    {
                        OnEmpty::Quiet => quiet_success(),
                        OnEmpty::Text(text) =>
                            success_message(repo_name, text.to_owned()),
                    }
                }
                else
                {
                    success_message(repo_name, line)
                }
            }
        })
    }
    else
    {
        let message = match failure
        {
            FailureReport::Line { from, fallback } =>
                from.first_line(output, fallback),
            FailureReport::Detailed { command } => format!(
                "git {command} failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            ),
            FailureReport::LastStderrLine { fallback } =>
                last_line(&output.stderr).unwrap_or_else(|| {
                    Streams::StdoutOnly.first_line(output, fallback)
                }),
        };

        Ok(failure_message(repo_name, message))
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

    fn output(code: i32, stdout: &str, stderr: &str) -> GitOutput
    {
        GitOutput {
            status: exit_status(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec()
        }
    }

    fn repo_path() -> &'static Path
    {
        Path::new("/vmr/backend")
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
            "backend",
            repo_path(),
            &output(0, "chatter\n", "chatter\n"),
            SuccessReport::Quiet,
            FailureReport::Line {
                from: Streams::StderrThenStdout,
                fallback: "failed"
            }
        )
        .unwrap();

        assert_eq!(success_text(outcome), None);
    }

    #[test]
    fn fixed_success_reports_the_exact_message()
    {
        let outcome = command_result(
            "backend",
            repo_path(),
            &output(0, "chatter\n", ""),
            SuccessReport::Fixed("Deleted branch topic".to_owned()),
            FailureReport::Line {
                from: Streams::StderrThenStdout,
                fallback: "failed"
            }
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
                "backend",
                repo_path(),
                &output(0, "", ""),
                SuccessReport::Line {
                    from: Streams::StdoutThenStderr,
                    on_empty
                },
                FailureReport::Line {
                    from: Streams::StderrThenStdout,
                    fallback: "failed"
                }
            )
            .unwrap();

            assert_eq!(success_text(outcome), expected.map(str::to_owned));
        }
    }

    #[test]
    fn line_success_reports_the_first_non_empty_line()
    {
        let outcome = command_result(
            "backend",
            repo_path(),
            &output(0, "\nfirst real line\nsecond line\n", ""),
            SuccessReport::Line {
                from: Streams::StdoutThenStderr,
                on_empty: OnEmpty::Quiet
            },
            FailureReport::Line {
                from: Streams::StderrThenStdout,
                fallback: "failed"
            }
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
                "backend",
                repo_path(),
                &output(1, "", stderr),
                SuccessReport::Quiet,
                FailureReport::Line {
                    from: Streams::StderrThenStdout,
                    fallback: "git push failed"
                }
            )
            .unwrap();

            assert_eq!(failure_text(outcome), expected);
        }
    }

    #[test]
    fn detailed_failure_formats_command_path_and_stderr()
    {
        let outcome = command_result(
            "backend",
            repo_path(),
            &output(1, "", "pathspec did not match\n"),
            SuccessReport::Quiet,
            FailureReport::Detailed { command: "add" }
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
                "backend",
                repo_path(),
                &output(1, stdout, stderr),
                SuccessReport::Quiet,
                FailureReport::LastStderrLine {
                    fallback: "git worktree add failed"
                }
            )
            .unwrap();

            assert_eq!(failure_text(outcome), expected);
        }
    }
}
