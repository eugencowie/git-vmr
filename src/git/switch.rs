use crate::git::{
    GitCommandResult, command_result, first_non_empty_line_with_fallback,
    git_output
};
use std::path::Path;

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
            Some(first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                "git switch succeeded"
            ))
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
