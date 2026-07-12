use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static BEHIND_COMMIT_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" by \d+ commits?").unwrap());

fn normalize_success_message(message: String) -> String
{
    BEHIND_COMMIT_COUNT.replace(&message, "").into_owned()
}

/// The report policy `switch` and `create` share: both run `git switch`,
/// so both normalize its fast-forward success message the same way.
fn report_policy() -> (SuccessReport, FailureReport)
{
    (
        SuccessReport::line(
            Streams::StdoutThenStderr,
            OnEmpty::Text("git switch succeeded")
        )
        .map(normalize_success_message),
        FailureReport::line(Streams::StderrThenStdout, "git switch failed")
    )
}

impl Git
{
    pub fn switch(
        &self,
        repo_name: &str,
        repo_path: &Path,
        branch_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["switch", branch_name])?;
        let (success, failure) = report_policy();

        command_result(repo_name, repo_path, &output, success, failure)
    }

    pub fn create(
        &self,
        repo_name: &str,
        repo_path: &Path,
        branch_name: &str
    ) -> GitCommandResult
    {
        let output =
            self.output(repo_path, ["switch", "--create", branch_name])?;
        let (success, failure) = report_policy();

        command_result(repo_name, repo_path, &output, success, failure)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;

    #[test]
    fn normalizes_behind_fast_forward_success_message()
    {
        assert_eq!(
            normalize_success_message(
                "Your branch is behind 'origin/develop' by 1 commit, and can be fast-forwarded."
                    .to_owned()
            ),
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
        assert_eq!(
            normalize_success_message(
                "Your branch is behind 'origin/develop' by 20 commits, and can be fast-forwarded."
                    .to_owned()
            ),
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
    }

    #[test]
    fn leaves_other_success_messages_unchanged()
    {
        assert_eq!(
            normalize_success_message(
                "Switched to branch 'feature/auth'".to_owned()
            ),
            "Switched to branch 'feature/auth'"
        );
    }

    #[test]
    fn switch_normalizes_behind_count_in_success_message()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["switch", "develop"],
            0,
            "Your branch is behind 'origin/develop' by 3 commits, and can be fast-forwarded.\n",
            ""
        ));

        // Act
        let outcome = git
            .switch("backend", Path::new("/vmr/backend"), "develop")
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(
            message.message,
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
    }

    #[test]
    fn create_normalizes_behind_count_in_success_message()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["switch", "--create", "feature/auth"],
            0,
            "Your branch is behind 'origin/develop' by 3 commits, and can be fast-forwarded.\n",
            ""
        ));

        // Act
        let outcome = git
            .create("backend", Path::new("/vmr/backend"), "feature/auth")
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(
            message.message,
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
    }
}
