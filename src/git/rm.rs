use crate::git::{
    GitCommandResult, command_result, first_non_empty_line, git_output, stderr
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn rm(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf],
    recursive: bool,
    force: bool,
    dry_run: bool,
    cached: bool
) -> GitCommandResult
{
    let mut args = vec![OsString::from("rm")];

    if recursive
    {
        args.push(OsString::from("-r"));
    }

    if force
    {
        args.push(OsString::from("--force"));
    }

    if dry_run
    {
        args.push(OsString::from("--dry-run"));
    }

    if cached
    {
        args.push(OsString::from("--cached"));
    }

    args.push(OsString::from("--"));
    args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
    let output = git_output(repo_path, args)?;

    command_result(
        repo_name,
        &output,
        |output| {
            if dry_run
            {
                let message = first_non_empty_line(&output.stdout, "");

                if message.is_empty() { None } else { Some(message) }
            }
            else
            {
                None
            }
        },
        |output| {
            format!(
                "git rm failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}
