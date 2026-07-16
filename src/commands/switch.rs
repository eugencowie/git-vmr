use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &SwitchArgs
) -> Result<Rendered>
{
    // Switch in each child repository, creating the branch when asked
    workspace.run(|git, repo| git.switch(repo, args))
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use regex::Regex;
use std::sync::LazyLock;

static BEHIND_COMMIT_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" by \d+ commits?").unwrap());

fn normalize_success_message(message: String) -> String
{
    BEHIND_COMMIT_COUNT.replace(&message, "").into_owned()
}

/// The switch report policy: normalize the fast-forward success message,
/// with or without `--create`.
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

/// Switch branches
#[derive(clap::Args)]
pub struct SwitchArgs
{
    /// Create a new branch named <branch> before switching to the branch
    #[arg(short, long)]
    pub create: bool,

    /// Branch to switch to
    #[arg(required = true, value_name = "branch")]
    pub branch_name: String
}

impl Git
{
    fn switch(&self, repo: &Repo, args: &SwitchArgs) -> GitCommandResult
    {
        let mut invocation = vec!["switch"];

        if args.create
        {
            invocation.push("--create");
        }

        invocation.push(&args.branch_name);

        let output = self.output(&repo.path, invocation)?;
        let (success, failure) = report_policy();

        command_result(repo, &output, success, failure)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{RepoOutcome, ScriptedFake};
    use crate::test_support::repo;

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
            .switch(&repo("backend", "/vmr/backend"), &SwitchArgs {
                create: false,
                branch_name: "develop".to_owned()
            })
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
            .switch(&repo("backend", "/vmr/backend"), &SwitchArgs {
                create: true,
                branch_name: "feature/auth".to_owned()
            })
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
