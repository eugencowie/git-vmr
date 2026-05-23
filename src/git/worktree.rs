use crate::git::{
    GitCommandResult, command_result, first_non_empty_line_with_fallback,
    git_output
};
use std::ffi::OsString;
use std::path::Path;

pub fn worktree_add(
    repo_name: &str,
    repo_path: &Path,
    target: &Path,
    branch: Option<&str>,
    commit_ish: Option<&str>
) -> GitCommandResult
{
    let mut args = vec![OsString::from("worktree"), OsString::from("add")];

    if let Some(branch) = branch
    {
        args.push(OsString::from("-b"));
        args.push(OsString::from(branch));
    }

    args.push(target.as_os_str().to_owned());

    if let Some(commit_ish) = commit_ish
    {
        args.push(OsString::from(commit_ish));
    }

    let output = git_output(repo_path, args)?;

    command_result(
        repo_name,
        &output,
        |output| {
            Some(first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git worktree add succeeded"
            ))
        },
        failure_line
    )
}

pub fn worktree_remove(
    repo_name: &str,
    repo_path: &Path,
    target: &Path,
    force: u8
) -> GitCommandResult
{
    let mut args = vec![OsString::from("worktree"), OsString::from("remove")];

    for _ in 0..force
    {
        args.push(OsString::from("-f"));
    }

    args.push(target.as_os_str().to_owned());

    let output = git_output(repo_path, args)?;

    command_result(
        repo_name,
        &output,
        |output| {
            let message = first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                "git worktree remove succeeded"
            );

            if message == "git worktree remove succeeded"
            {
                None
            }
            else
            {
                Some(message)
            }
        },
        |output| {
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git worktree remove failed"
            )
        }
    )
}

fn failure_line(output: &crate::git::GitOutput) -> String
{
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                "git worktree add failed"
            )
        })
}
