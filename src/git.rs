mod branch;
mod head;
pub(crate) mod report;
mod runner;

use anyhow::{Result, bail};
pub use head::Head;
#[cfg(test)]
pub(crate) use runner::scripted::{Invocation, ScriptedFake};
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
            .chain(paths.into_iter().map(|path| git_path_arg(path.as_ref())))
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
    let path = path.display().to_string().replace('\\', "/");
    if let Some(path) = path.strip_prefix("//?/UNC/")
    {
        format!("//{path}")
    }
    else
    {
        path.strip_prefix("//?/").unwrap_or(&path).to_owned()
    }
}

/// A filesystem path passed to Git as a pathspec or repository-relative
/// operand. Git for Windows accepts forward slashes and reports paths in that
/// form; preserve Unix backslashes, where they are valid filename bytes.
pub(crate) fn git_path_arg(path: &Path) -> OsString
{
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let wide = windows_git_path(path.as_os_str().encode_wide().collect());
        OsString::from_wide(&wide)
    }

    #[cfg(not(windows))]
    {
        path.as_os_str().to_owned()
    }
}

/// Removes Windows' verbatim prefix, which Git for Windows does not accept,
/// and uses the forward-slash spelling Git emits. Kept platform-neutral so
/// the transformation remains directly testable off Windows.
#[cfg(any(windows, test))]
fn windows_git_path(wide: Vec<u16>) -> Vec<u16>
{
    const VERBATIM: &[u16] =
        &[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    const VERBATIM_UNC: &[u16] = &[
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'U' as u16,
        b'N' as u16,
        b'C' as u16,
        b'\\' as u16
    ];

    let (prefix, path) = if let Some(path) = wide.strip_prefix(VERBATIM_UNC)
    {
        (&[b'/' as u16, b'/' as u16][..], path)
    }
    else if let Some(path) = wide.strip_prefix(VERBATIM)
    {
        (&[][..], path)
    }
    else
    {
        (&[][..], wide.as_slice())
    };

    prefix
        .iter()
        .copied()
        .chain(path.iter().copied().map(|unit| {
            if unit == u16::from(b'\\') { u16::from(b'/') } else { unit }
        }))
        .collect()
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
    fn git_style_path_removes_windows_verbatim_prefix()
    {
        assert_eq!(
            git_style_path(Path::new(r"\\?\C:\Projects\vmr")),
            "C:/Projects/vmr"
        );
    }

    #[test]
    fn git_path_arg_removes_windows_verbatim_prefix()
    {
        let wide = r"\\?\C:\Projects\vmr".encode_utf16().collect();
        let normalized = String::from_utf16(&windows_git_path(wide)).unwrap();

        assert_eq!(normalized, "C:/Projects/vmr");
    }

    #[test]
    fn git_path_arg_preserves_windows_unc_root()
    {
        let wide = r"\\?\UNC\server\share\vmr".encode_utf16().collect();
        let normalized = String::from_utf16(&windows_git_path(wide)).unwrap();

        assert_eq!(normalized, "//server/share/vmr");
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
