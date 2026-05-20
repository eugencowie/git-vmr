use crate::git::{
    GitCommandResult, command_result, first_non_empty_line_with_fallback,
    git_output
};
use std::path::Path;

pub fn rebase(
    repo_name: &str,
    repo_path: &Path,
    upstream: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["rebase", upstream])?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git rebase failed"
            )
        }
    )
}
