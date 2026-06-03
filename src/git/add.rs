use crate::git::{GitCommandResult, command_result, git_path_output, stderr};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub fn add(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf]
) -> GitCommandResult
{
    let output = git_path_output(repo_path, ["add", "--"], paths)?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            format!(
                "git add failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}

pub fn add_path(repo_path: &Path, path: &Path) -> Result<()>
{
    let output = git_path_output(repo_path, ["add", "--"], [path])?;

    if !output.status.success()
    {
        bail!(
            "git add failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}
