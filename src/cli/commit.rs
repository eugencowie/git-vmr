use crate::cli::AggregateError;
use crate::config::vmr;
use anyhow::{Context, Result, bail};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

struct CommitSuccess
{
    repo_name: String,
    stdout: String
}

struct CommitFailure
{
    repo_name: String,
    stderr: String
}

struct GitOutput
{
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>
}

pub fn commit(working_dir: &Path, message: &str) -> Result<()>
{
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let repos = eligible_repos(&vmr_root)?;

    if repos.is_empty()
    {
        bail!("nothing to commit");
    }

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for (repo_name, repo_path) in repos
    {
        match commit_repo(&repo_name, &repo_path, message)?
        {
            Ok(success) => successes.push(success),
            Err(failure) => failures.push(failure)
        }
    }

    for success in successes
    {
        println!("{} ({})", success.stdout, success.repo_name);
    }

    if !failures.is_empty()
    {
        return Err(AggregateError::new(
            failures
                .into_iter()
                .map(|failure| {
                    anyhow::anyhow!(
                        "{} ({})",
                        failure.stderr,
                        failure.repo_name
                    )
                })
                .collect()
        )
        .into());
    }

    Ok(())
}

fn eligible_repos(vmr_root: &Path) -> Result<Vec<(String, PathBuf)>>
{
    let mut repos = Vec::new();

    for repo_path in child_dirs(vmr_root)?
    {
        if !repo_path.join(".git").exists()
        {
            continue;
        }

        let repo_name = repo_path
            .file_name()
            .and_then(|name| name.to_str())
            .context("repository path has no valid UTF-8 file name")?
            .to_owned();

        if has_staged_changes(&repo_path)?
        {
            repos.push((repo_name, repo_path));
        }
    }

    repos.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(repos)
}

fn child_dirs(vmr_root: &Path) -> Result<Vec<PathBuf>>
{
    Ok(fs::read_dir(vmr_root)
        .with_context(|| {
            format!("failed to read VMR root '{}'", vmr_root.display())
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false)
        })
        .filter(|entry| entry.file_name() != OsStr::new(".gitvmr"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>())
}

fn has_staged_changes(repo_path: &Path) -> Result<bool>
{
    let output = git_output(repo_path, &["diff", "--cached", "--quiet"])
        .with_context(|| {
            format!(
                "failed to inspect staged changes for '{}'",
                repo_path.display()
            )
        })?;

    match output.status.code()
    {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => bail!(
            "git diff --cached --quiet failed for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}

fn commit_repo(
    repo_name: &str,
    repo_path: &Path,
    message: &str
) -> Result<std::result::Result<CommitSuccess, CommitFailure>>
{
    let output = git_output(repo_path, &["commit", "-m", message])?;

    if output.status.success()
    {
        return Ok(Ok(CommitSuccess {
            repo_name: repo_name.to_owned(),
            stdout: first_non_empty_line(
                &output.stdout,
                "git commit succeeded"
            )
        }));
    }

    Ok(Err(CommitFailure {
        repo_name: repo_name.to_owned(),
        stderr: first_non_empty_line(&output.stderr, "git commit failed")
    }))
}

fn git_output(repo_path: &Path, args: &[&str]) -> Result<GitOutput>
{
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .args(args)
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

fn first_non_empty_line(bytes: &[u8], fallback: &str) -> String
{
    let text = String::from_utf8_lossy(bytes);
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(fallback)
        .to_owned()
}
