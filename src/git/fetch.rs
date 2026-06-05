use crate::git::{
    GitCommandResult, command_result, first_non_empty_line_with_fallback,
    git_output
};
use std::ffi::OsString;
use std::path::Path;

pub fn fetch(
    repo_name: &str,
    repo_path: &Path,
    repository: Option<&str>,
    refspecs: &[String]
) -> GitCommandResult
{
    let mut args = vec![OsString::from("fetch")];

    if let Some(repository) = repository
    {
        args.push(OsString::from(repository));
    }

    args.extend(refspecs.iter().map(OsString::from));

    let output = git_output(repo_path, args)?;

    command_result(
        repo_name,
        &output,
        |output| {
            let message = first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                ""
            );

            if message.is_empty() { None } else { Some(message) }
        },
        |output| {
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git fetch failed"
            )
        }
    )
}
