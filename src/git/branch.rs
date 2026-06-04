use crate::git::{
    GitCommandResult, GitOutput, Head, RepoBranches, command_result,
    first_non_empty_line_strip_fatal, git_output, git_stdout
};
use crate::vmr::Repo;
use anyhow::{Context, Result, bail};
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

pub fn delete_branch(
    repo_name: &str,
    repo_path: &Path,
    branch_name: &str,
    force: bool
) -> GitCommandResult
{
    let flag = if force { "-D" } else { "-d" };
    let output = git_output(repo_path, ["branch", flag, branch_name])?;

    command_result(
        repo_name,
        &output,
        |output| {
            first_non_empty_line_strip_fatal(
                &output.stdout,
                "git branch failed"
            )
            .into()
        },
        |output| {
            first_non_empty_line_strip_fatal(
                &output.stderr,
                "git branch failed"
            )
        }
    )
}

pub fn branches(repo: &Repo) -> Result<Option<(String, RepoBranches)>>
{
    let branches_output = git_stdout(&repo.path, [
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/heads"
    ])
    .with_context(|| {
        format!(
            "failed to read branch information for '{}'",
            repo.path.display()
        )
    })?;
    let branches = String::from_utf8_lossy(&branches_output)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let head = match git_output(&repo.path, [
        "symbolic-ref",
        "--quiet",
        "--short",
        "HEAD"
    ])
    .with_context(|| {
        format!(
            "failed to read branch information for '{}'",
            repo.path.display()
        )
    })?
    {
        GitOutput { status, stdout, stderr: _ } if status.success() =>
            Head::Branch(String::from_utf8_lossy(&stdout).trim().to_owned()),
        GitOutput { status, .. } if status.code() == Some(1) =>
        {
            let hash = String::from_utf8_lossy(
                &git_stdout(&repo.path, ["rev-parse", "--short", "HEAD"])
                    .with_context(|| {
                        format!(
                            "failed to read branch information for '{}'",
                            repo.path.display()
                        )
                    })?
            )
            .trim()
            .to_owned();
            Head::Detached(hash)
        }
        GitOutput { stderr, .. } => bail!(
            "failed to read branch information for '{}': {}",
            repo.path.display(),
            String::from_utf8_lossy(&stderr).trim()
        )
    };

    Ok(Some((repo.name.clone(), RepoBranches { branches, head })))
}
