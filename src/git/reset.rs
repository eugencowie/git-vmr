use crate::git::{
    GitCommandResult, command_result, first_non_empty_line_with_fallback,
    git_output
};
use std::ffi::OsString;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetMode
{
    Soft,
    Mixed,
    Hard,
    Merge,
    Keep
}

impl ResetMode
{
    pub fn as_arg(self) -> &'static str
    {
        match self
        {
            ResetMode::Soft => "--soft",
            ResetMode::Mixed => "--mixed",
            ResetMode::Hard => "--hard",
            ResetMode::Merge => "--merge",
            ResetMode::Keep => "--keep"
        }
    }
}

pub fn reset(
    repo_name: &str,
    repo_path: &Path,
    mode: Option<ResetMode>,
    commit: Option<&str>
) -> GitCommandResult
{
    let mut args = vec![OsString::from("reset")];

    if let Some(mode) = mode
    {
        args.push(OsString::from(mode.as_arg()));
    }

    if let Some(commit) = commit
    {
        args.push(OsString::from(commit));
    }

    let output = git_output(repo_path, args)?;

    command_result(
        repo_name,
        &output,
        |output| {
            let message = first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                ""
            );

            if message.is_empty() { None } else { Some(message) }
        },
        |output| {
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git reset failed"
            )
        }
    )
}
