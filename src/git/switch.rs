use crate::git::{
    GitCommandResult, command_result, failure_message,
    first_non_empty_line_with_fallback, git_output, success_message
};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static BEHIND_COMMIT_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" by \d+ commits?").unwrap());

pub fn switch(
    repo_name: &str,
    repo_path: &Path,
    branch_name: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["switch", branch_name])?;

    command_result(
        repo_name,
        &output,
        |output| {
            Some(normalize_success_message(first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                "git switch succeeded"
            )))
        },
        |output| {
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git switch failed"
            )
        }
    )
}

fn normalize_success_message(message: String) -> String
{
    BEHIND_COMMIT_COUNT.replace(&message, "").into_owned()
}

pub fn create(
    repo_name: &str,
    repo_path: &Path,
    branch_name: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["switch", "--create", branch_name])?;

    if output.status.success()
    {
        Ok(success_message(
            repo_name,
            first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                "git switch succeeded"
            )
        ))
    }
    else
    {
        Ok(failure_message(
            repo_name,
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git switch failed"
            )
        ))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

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
}
