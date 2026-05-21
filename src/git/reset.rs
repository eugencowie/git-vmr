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
    pub fn from_arg(
        soft: bool,
        mixed: bool,
        hard: bool,
        merge: bool,
        keep: bool
    ) -> Option<ResetMode>
    {
        match (soft, mixed, hard, merge, keep)
        {
            (true, false, false, false, false) => Some(ResetMode::Soft),
            (false, true, false, false, false) => Some(ResetMode::Mixed),
            (false, false, true, false, false) => Some(ResetMode::Hard),
            (false, false, false, true, false) => Some(ResetMode::Merge),
            (false, false, false, false, true) => Some(ResetMode::Keep),
            (false, false, false, false, false) => None,
            _ => unreachable!()
        }
    }

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn from_arg_returns_none_when_no_mode_flag_is_set()
    {
        // Act
        let mode = ResetMode::from_arg(false, false, false, false, false);

        // Assert
        assert_eq!(mode, None);
    }

    #[test]
    fn from_arg_returns_selected_mode()
    {
        for (flags, expected) in [
            ((true, false, false, false, false), ResetMode::Soft),
            ((false, true, false, false, false), ResetMode::Mixed),
            ((false, false, true, false, false), ResetMode::Hard),
            ((false, false, false, true, false), ResetMode::Merge),
            ((false, false, false, false, true), ResetMode::Keep)
        ]
        {
            // Act
            let mode = ResetMode::from_arg(
                flags.0, flags.1, flags.2, flags.3, flags.4
            );

            // Assert
            assert_eq!(mode, Some(expected));
        }
    }
}
