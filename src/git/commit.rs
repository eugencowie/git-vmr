use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult, stderr};
use crate::vmr::Repo;
use anyhow::{Context, Result, bail};
use std::path::Path;

/// Record changes to the repositories
#[derive(clap::Args)]
pub struct CommitArgs
{
    /// Use <msg> as the commit message
    #[arg(short, long, required = true, value_name = "msg")]
    pub message: String
}

impl Git
{
    pub fn commit(&self, repo: &Repo, args: &CommitArgs) -> GitCommandResult
    {
        let output =
            self.output(&repo.path, ["commit", "-m", &args.message])?;

        command_result(
            repo,
            &output,
            SuccessReport::line(
                Streams::StdoutOnly,
                OnEmpty::Text("git commit succeeded")
            ),
            FailureReport::line(Streams::StderrOnly, "git commit failed")
        )
    }

    pub fn is_dirty(&self, repo_path: &Path) -> Result<bool>
    {
        let output = self
            .output(repo_path, ["diff", "--cached", "--quiet"])
            .with_context(|| {
                format!(
                    "fatal: failed to inspect staged changes for '{}'",
                    repo_path.display()
                )
            })?;

        match output.status.code()
        {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => bail!(
                "fatal: git diff --cached --quiet failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            )
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;
    use crate::test_support::repo;

    #[test]
    fn is_dirty_maps_diff_exit_codes()
    {
        for (code, expected) in [(0, false), (1, true)]
        {
            // Arrange
            let git = Git::with(ScriptedFake::new().on(
                ["diff", "--cached", "--quiet"],
                code,
                "",
                ""
            ));

            // Act
            let dirty = git.is_dirty(Path::new("/vmr/backend")).unwrap();

            // Assert
            assert_eq!(dirty, expected);
        }
    }

    #[test]
    fn commit_reports_first_stdout_line_through_the_seam()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["commit", "-m", "message"],
            0,
            "[main abc1234] message\n 1 file changed\n",
            ""
        ));

        // Act
        let outcome = git
            .commit(&repo("backend", "/vmr/backend"), &CommitArgs {
                message: "message".to_owned()
            })
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.repo, "backend");
        assert_eq!(message.message, "[main abc1234] message");
    }

    #[test]
    fn commit_reports_first_stderr_line_on_failure()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["commit", "-m", "message"],
            1,
            "",
            "error: nothing to commit\n"
        ));

        // Act
        let outcome = git
            .commit(&repo("backend", "/vmr/backend"), &CommitArgs {
                message: "message".to_owned()
            })
            .unwrap();

        // Assert
        let RepoOutcome::Failure(message) = outcome
        else
        {
            panic!("expected failure");
        };
        assert_eq!(message.message, "error: nothing to commit");
    }
}
