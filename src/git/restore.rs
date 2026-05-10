use crate::git::{GitCommandResult, command_result, git_output, stderr};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn restore(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> GitCommandResult
{
    let mut args = vec![OsString::from("restore")];

    if staged
    {
        args.push(OsString::from("--staged"));
    }

    if worktree
    {
        args.push(OsString::from("--worktree"));
    }

    args.push(OsString::from("--"));
    args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
    let output = git_output(repo_path, args)?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            format!(
                "git restore failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}
