use crate::cli::AggregateError;
use crate::config::vmr;
use crate::git::{self, git_output};
use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

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

    for result in repos
        .par_iter()
        .map(|(repo_name, repo_path)| {
            git::commit(repo_name, repo_path, message)
        })
        .collect::<Result<Vec<_>>>()?
    {
        match result
        {
            Ok(success) => successes.push(success),
            Err(failure) => failures.push(failure)
        }
    }

    successes.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));
    failures.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

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
                        failure.message,
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
