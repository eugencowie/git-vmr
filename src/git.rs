mod add;
mod branch;
mod commit;
mod merge;
mod mv;
mod rebase;
mod restore;
mod rm;

pub use add::{add, add_path};
use anyhow::{Context, Result, anyhow, bail};
pub use branch::branch;
pub use commit::commit;
pub use merge::merge;
pub use mv::{ensure_tracked, mv};
pub use rebase::rebase;
pub use restore::restore;
pub use rm::rm;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, ExitStatus};

pub struct GitOutput
{
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>
}

pub type GitCommandResult = Result<Option<String>>;

pub(crate) fn command_result(
    repo_name: &str,
    output: &GitOutput,
    success_message: impl FnOnce(&GitOutput) -> Option<String>,
    failure_message: impl FnOnce(&GitOutput) -> String
) -> GitCommandResult
{
    if output.status.success()
    {
        Ok(success_message(output)
            .map(|message| format!("{message} ({repo_name})")))
    }
    else
    {
        Err(anyhow!("{} ({})", failure_message(output), repo_name))
    }
}

pub fn git_output<I, S>(repo_path: &Path, args: I) -> Result<GitOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    let args =
        args.into_iter().map(|arg| arg.as_ref().to_owned()).collect::<Vec<_>>();

    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .args(&args)
        .output()
        .with_context(|| {
            format!("failed to invoke git for '{}'", repo_path.display())
        })?;

    Ok(GitOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr
    })
}

pub fn git_stdout<I, S>(repo_path: &Path, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    let args =
        args.into_iter().map(|arg| arg.as_ref().to_owned()).collect::<Vec<_>>();
    let args_display = format_git_args(&args);
    let output = git_output(repo_path, args)?;

    if !output.status.success()
    {
        bail!(
            "git {} failed for '{}': {}",
            args_display,
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(output.stdout)
}

pub(crate) fn git_path_output<I, S, P>(
    repo_path: &Path,
    args: I,
    paths: P
) -> Result<GitOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    P: IntoIterator,
    P::Item: AsRef<Path>
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .chain(
            paths.into_iter().map(|path| path.as_ref().as_os_str().to_owned())
        )
        .collect::<Vec<_>>();
    git_output(repo_path, args)
}

pub(crate) fn stderr(output: &GitOutput) -> String
{
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

fn format_git_args(args: &[OsString]) -> String
{
    args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ")
}

pub(crate) fn first_non_empty_line(bytes: &[u8], fallback: &str) -> String
{
    let text = String::from_utf8_lossy(bytes);
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(fallback)
        .to_owned()
}

pub(crate) fn first_non_empty_line_strip_fatal(
    bytes: &[u8],
    fallback: &str
) -> String
{
    let text = String::from_utf8_lossy(bytes);
    let line =
        text.lines().find(|line| !line.trim().is_empty()).unwrap_or(fallback);

    line.strip_prefix("fatal: ").unwrap_or(line).to_owned()
}

pub(crate) fn first_non_empty_line_with_fallback(
    primary: &[u8],
    secondary: &[u8],
    fallback: &str
) -> String
{
    let primary_line = first_non_empty_line(primary, "");

    if primary_line.is_empty()
    {
        first_non_empty_line(secondary, fallback)
    }
    else
    {
        primary_line
    }
}

pub(crate) fn first_non_empty_line_with_fallback_strip_fatal(
    primary: &[u8],
    secondary: &[u8],
    fallback: &str
) -> String
{
    let primary_line = first_non_empty_line_strip_fatal(primary, "");

    if primary_line.is_empty()
    {
        first_non_empty_line_strip_fatal(secondary, fallback)
    }
    else
    {
        primary_line
    }
}
