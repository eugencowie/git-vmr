mod add;
mod branch;
mod commit;
mod diff;
mod fetch;
mod merge;
mod mv;
mod pull;
mod push;
mod rebase;
mod reset;
mod restore;
mod rm;
mod status;
mod switch;
mod tag;

pub use add::{add, add_path};
use anyhow::{Context, Result, anyhow, bail};
pub use branch::{branch, branches, delete_branch};
pub use commit::commit;
pub use diff::is_dirty;
pub use fetch::fetch;
pub use merge::merge;
pub use mv::{ensure_tracked, mv};
pub use pull::pull;
pub use push::push;
pub use rebase::rebase;
pub use reset::{ResetMode, reset};
pub use restore::restore;
pub use rm::rm;
pub use status::status;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
pub use switch::{create, switch};
pub use tag::{delete_tag, tag, tags};

#[derive(Clone, PartialEq, Eq)]
pub enum Head
{
    Branch(String),
    Detached(String)
}

#[derive(Clone, PartialEq, Eq)]
pub struct RepoBranches
{
    pub branches: Vec<String>,
    pub head: Head
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileChange
{
    NewFile,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Copied
}

#[derive(Clone, PartialEq, Eq)]
pub struct FileEntry
{
    pub path: PathBuf,
    pub change: FileChange
}

#[derive(Clone, PartialEq, Eq)]
pub struct RepoStatus
{
    pub head: Head,
    pub initial: bool,
    pub staged_changes: Vec<FileEntry>,
    pub unstaged_changes: Vec<FileEntry>,
    pub untracked_files: Vec<FileEntry>
}

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

pub(crate) fn status_head(
    repo_path: &Path,
    context: &str,
    header: &[u8]
) -> Result<(Head, bool)>
{
    // Decode and validate branch header.
    let header = String::from_utf8_lossy(header);
    let header = header
        .strip_prefix("## ")
        .context("git status branch header had unexpected format")?;

    if let Some(branch) = header.strip_prefix("No commits yet on ")
    {
        return Ok((Head::Branch(branch.to_owned()), true));
    }

    if header == "HEAD (no branch)" || header.starts_with("HEAD detached")
    {
        let hash = String::from_utf8_lossy(
            &git_stdout(repo_path, ["rev-parse", "--short", "HEAD"])
                .with_context(|| {
                    format!("{context} for '{}'", repo_path.display())
                })?
        )
        .trim()
        .to_owned();
        return Ok((Head::Detached(hash), false));
    }

    let branch = header.split("...").next().unwrap_or(header).to_owned();
    Ok((Head::Branch(branch), false))
}
