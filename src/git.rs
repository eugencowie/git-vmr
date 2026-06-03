use anyhow::{Context, Result, anyhow, bail};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

pub struct GitOutput
{
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>
}

pub type GitCommandResult = Result<Option<String>>;

pub fn add(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf]
) -> GitCommandResult
{
    let output = git_path_output(repo_path, ["add", "--"], paths)?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            format!(
                "git add failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}

pub fn branch(
    repo_name: &str,
    repo_path: &Path,
    branch_name: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["branch", branch_name])?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            first_non_empty_line_strip_fatal(
                &output.stderr,
                "git branch failed"
            )
        }
    )
}

pub fn commit(
    repo_name: &str,
    repo_path: &Path,
    message: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["commit", "-m", message])?;

    command_result(
        repo_name,
        &output,
        |output| {
            Some(first_non_empty_line(&output.stdout, "git commit succeeded"))
        },
        |output| first_non_empty_line(&output.stderr, "git commit failed")
    )
}

pub fn merge(
    repo_name: &str,
    repo_path: &Path,
    commit_ish: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["merge", commit_ish])?;

    command_result(
        repo_name,
        &output,
        |output| {
            Some(first_non_empty_line(&output.stdout, "git merge succeeded"))
        },
        |output| {
            first_non_empty_line_with_fallback(
                &output.stderr,
                &output.stdout,
                "git merge failed"
            )
        }
    )
}

pub fn rebase(
    repo_name: &str,
    repo_path: &Path,
    upstream: &str
) -> GitCommandResult
{
    let output = git_output(repo_path, ["rebase", upstream])?;

    command_result(
        repo_name,
        &output,
        |_| None,
        |output| {
            first_non_empty_line_with_fallback_strip_fatal(
                &output.stderr,
                &output.stdout,
                "git rebase failed"
            )
        }
    )
}

pub fn restore(
    repo_name: &str,
    repo_path: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> GitCommandResult
{
    let mut args = vec![OsString::from("restore")];

    if staged
    {
        args.push(OsString::from("--staged"));
    }

    if worktree
    {
        args.push(OsString::from("--worktree"));
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
                "git restore failed for '{}': {}",
                repo_path.display(),
                stderr(output)
            )
        }
    )
}

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

pub fn mv(repo_path: &Path, source: &Path, destination: &Path) -> Result<()>
{
    let output =
        git_path_output(repo_path, ["mv", "--"], [source, destination])?;

    if !output.status.success()
    {
        bail!(
            "git mv failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}

pub fn ensure_tracked(repo_path: &Path, path: &Path) -> Result<()>
{
    let output =
        git_path_output(repo_path, ["ls-files", "--error-unmatch", "--"], [
            path
        ])?;

    if !output.status.success()
    {
        bail!(
            "source path is not tracked in '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}

pub fn add_path(repo_path: &Path, path: &Path) -> Result<()>
{
    let output = git_path_output(repo_path, ["add", "--"], [path])?;

    if !output.status.success()
    {
        bail!(
            "git add failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}

fn command_result(
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

fn git_path_output<I, S, P>(
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

fn stderr(output: &GitOutput) -> String
{
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

fn format_git_args(args: &[OsString]) -> String
{
    args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ")
}

fn first_non_empty_line(bytes: &[u8], fallback: &str) -> String
{
    let text = String::from_utf8_lossy(bytes);
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(fallback)
        .to_owned()
}

fn first_non_empty_line_strip_fatal(bytes: &[u8], fallback: &str) -> String
{
    let text = String::from_utf8_lossy(bytes);
    let line =
        text.lines().find(|line| !line.trim().is_empty()).unwrap_or(fallback);

    line.strip_prefix("fatal: ").unwrap_or(line).to_owned()
}

fn first_non_empty_line_with_fallback(
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

fn first_non_empty_line_with_fallback_strip_fatal(
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
