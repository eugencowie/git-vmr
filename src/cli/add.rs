use crate::cli::AggregateError;
use crate::config::vmr;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

struct AddFailure
{
    repo_name: String,
    message: String
}

pub fn add(working_dir: &Path, paths: &[PathBuf]) -> Result<()>
{
    // Find VMR root and route requested paths
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let routed = vmr::route_paths(working_dir, &vmr_root, paths)?
        .into_iter()
        .collect::<Vec<_>>();

    // Stage paths in each child repository
    let mut failures = routed
        .par_iter()
        .filter_map(|(repo_path, repo_paths)| {
            git_add(repo_path, repo_paths).transpose()
        })
        .collect::<Result<Vec<_>>>()?;

    failures.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

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

fn git_add(repo_path: &Path, paths: &[PathBuf]) -> Result<Option<AddFailure>>
{
    // Run git add in the owning child repository
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .arg("add")
        .arg("--")
        .args(paths)
        .output()
        .with_context(|| {
            format!("failed to invoke git for '{}'", repo_path.display())
        })?;

    // Convert git failure into anyhow error
    if !output.status.success()
    {
        return Ok(Some(AddFailure {
            repo_name: repo_name(repo_path)?,
            message: format!(
                "git add failed for '{}': {}",
                repo_path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            )
        }));
    }

    Ok(None)
}

fn repo_name(repo_path: &Path) -> Result<String>
{
    Ok(repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("repository path has no valid UTF-8 file name")?
        .to_owned())
}
