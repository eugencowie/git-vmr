mod branch;
mod head;
pub(crate) mod report;
mod runner;

use anyhow::{Result, bail};
pub use head::Head;
#[cfg(test)]
pub(crate) use runner::scripted::ScriptedFake;
pub use runner::{GitRunner, SubprocessRunner};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::ExitStatus;

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

    pub(crate) fn interactive(
        &self,
        working_dir: &Path,
        args: &[OsString]
    ) -> Result<ExitStatus>
    {
        self.runner.run_interactive(working_dir, args)
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
    Failure(RepoMessage),
    /// The operation was deliberately not attempted; the reason is required.
    Skipped(RepoMessage)
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

pub(crate) fn skip_message(repo_name: &str, reason: String) -> RepoOutcome
{
    RepoOutcome::Skipped(RepoMessage {
        repo: repo_name.to_owned(),
        message: reason
    })
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
