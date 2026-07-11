mod add;
mod branch;
mod clone;
mod commit;
mod fetch;
mod merge;
mod mv;
mod pull;
mod push;
mod rebase;
mod report;
mod reset;
mod restore;
mod rm;
mod runner;
mod status;
mod switch;
mod tag;
mod worktree;

pub use add::ChmodMode;
use anyhow::{Context, Result, bail};
pub use reset::ResetMode;
#[cfg(test)]
pub(crate) use runner::scripted::ScriptedFake;
pub use runner::{GitRunner, SubprocessRunner};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
pub use worktree::ChildWorktreeState;

/// The deep git module: every operation is a method, and every invocation
/// flows through the [`GitRunner`] seam owned here.
pub struct Git
{
    runner: Box<dyn GitRunner>
}

impl Git
{
    pub fn subprocess() -> Self
    {
        Self { runner: Box::new(SubprocessRunner) }
    }

    #[cfg(test)]
    pub(crate) fn with(runner: impl GitRunner + 'static) -> Self
    {
        Self { runner: Box::new(runner) }
    }

    pub(crate) fn output<I, S>(
        &self,
        repo_path: &Path,
        args: I
    ) -> Result<GitOutput>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        let args = args
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect::<Vec<_>>();

        self.runner.run_captured(repo_path, &args)
    }

    pub(crate) fn stdout<I, S>(
        &self,
        repo_path: &Path,
        args: I
    ) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        let args = args
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect::<Vec<_>>();
        let args_display = format_git_args(&args);
        let output = self.runner.run_captured(repo_path, &args)?;

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

    pub(crate) fn path_output<I, S, P>(
        &self,
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
                paths
                    .into_iter()
                    .map(|path| path.as_ref().as_os_str().to_owned())
            )
            .collect::<Vec<_>>();
        self.output(repo_path, args)
    }

    pub(crate) fn status_head(
        &self,
        repo_path: &Path,
        context: &str,
        header: &[u8]
    ) -> Result<(Head, bool)>
    {
        // Decode and validate branch header.
        let header = String::from_utf8_lossy(header);
        let header = header
            .strip_prefix("## ")
            .context("fatal: git status branch header had unexpected format")?;

        if let Some(branch) = header.strip_prefix("No commits yet on ")
        {
            return Ok((Head::Branch(branch.to_owned()), true));
        }

        if header == "HEAD (no branch)" || header.starts_with("HEAD detached")
        {
            let hash = String::from_utf8_lossy(
                &self
                    .stdout(repo_path, ["rev-parse", "--short", "HEAD"])
                    .with_context(|| {
                        format!(
                            "fatal: {context} for '{}'",
                            repo_path.display()
                        )
                    })?
            )
            .trim()
            .to_owned();
            return Ok((Head::Detached(hash), false));
        }

        let branch = header.split("...").next().unwrap_or(header).to_owned();
        Ok((Head::Branch(branch), false))
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileChange
{
    NewFile,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Copied
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

pub struct RepoMessage
{
    pub repo: String,
    pub message: String
}

pub enum RepoOutcome
{
    Success(Option<RepoMessage>),
    Failure(RepoMessage)
}

impl RepoOutcome
{
    /// Applies `map` to the success message, if there is one; failures and
    /// quiet successes pass through unchanged.
    pub(crate) fn map_success_message(
        self,
        map: impl FnOnce(String) -> String
    ) -> Self
    {
        match self
        {
            Self::Success(Some(message)) => Self::Success(Some(RepoMessage {
                repo: message.repo,
                message: map(message.message)
            })),
            other => other
        }
    }
}

pub type GitCommandResult = Result<RepoOutcome>;

pub(crate) fn git_style_path(path: &Path) -> String
{
    path.display().to_string().replace('\\', "/")
}

pub(crate) fn quiet_success() -> RepoOutcome
{
    RepoOutcome::Success(None)
}

pub(crate) fn success_message(repo_name: &str, message: String) -> RepoOutcome
{
    RepoOutcome::Success(Some(RepoMessage {
        repo: repo_name.to_owned(),
        message
    }))
}

pub(crate) fn failure_message(repo_name: &str, message: String) -> RepoOutcome
{
    RepoOutcome::Failure(RepoMessage { repo: repo_name.to_owned(), message })
}

pub(crate) fn stderr(output: &GitOutput) -> String
{
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

fn format_git_args(args: &[OsString]) -> String
{
    args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::Path;

    #[test]
    fn git_style_path_normalizes_windows_separators()
    {
        assert_eq!(
            git_style_path(Path::new(r"C:\Projects\vmr")),
            "C:/Projects/vmr"
        );
    }

    #[test]
    fn git_style_path_normalizes_mixed_separator_inputs()
    {
        assert_eq!(
            git_style_path(Path::new(r"C:\Projects\vmr")),
            "C:/Projects/vmr"
        );
        assert_eq!(
            git_style_path(Path::new("C:/Worktrees/new-feature")),
            "C:/Worktrees/new-feature"
        );
    }
}
