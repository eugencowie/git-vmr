use crate::git::{
    GitCommandResult, command_result, first_non_empty_line_strip_fatal,
    git_output
};
use std::path::Path;

pub fn branch(
    repo_name: &str,
    repo_path: &Path,
    branch_name: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["branch", branch_name])?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            first_non_empty_line_strip_fatal(
                &output.stderr,
                "git branch failed"
            )
        }
    )
}
