use crate::git::{
    GitCommandResult, command_result, first_non_empty_line, git_output,
    git_stdout
};
use crate::vmr::Repo;
use anyhow::{Context, Result};
use std::path::Path;

pub fn tag(
    repo_name: &str,
    repo_path: &Path,
    tag_name: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["tag", tag_name])?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| first_non_empty_line(&output.stderr, "git tag failed")
    )
}

pub fn delete_tag(
    repo_name: &str,
    repo_path: &Path,
    tag_name: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["tag", "-d", tag_name])?;

    command_result(
        repo_name,
        &output,
        |output| first_non_empty_line(&output.stdout, "git tag failed").into(),
        |output| first_non_empty_line(&output.stderr, "git tag failed")
    )
}

pub fn tags(repo: &Repo) -> Result<Option<(String, Vec<String>)>>
{
    let tags_output = git_stdout(&repo.path, [
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/tags"
    ])
    .with_context(|| {
        format!("failed to read tag information for '{}'", repo.path.display())
    })?;
    let tags = String::from_utf8_lossy(&tags_output)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    Ok(Some((repo.name.clone(), tags)))
}
