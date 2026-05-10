use crate::git::{GitCommandResult, command_result, git_output, stderr};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn rm(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf],
    recursive: bool
) -> GitCommandResult
{
    let mut args = vec![OsString::from("rm")];

    if recursive
    {
        args.push(OsString::from("-r"));
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
                "git rm failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}
