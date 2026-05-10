use crate::git::{
    GitCommandResult, command_result, first_non_empty_line, git_output
};
use std::path::Path;

pub fn commit(
    repo_name: &str,
    repo_path: &Path,
    message: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["commit", "-m", message])?;

    command_result(
        repo_name,
        &output,
        |output| {
            Some(first_non_empty_line(&output.stdout, "git commit succeeded"))
        },
        |output| first_non_empty_line(&output.stderr, "git commit failed")
    )
}
