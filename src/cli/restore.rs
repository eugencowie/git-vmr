use crate::cli::AggregateError;
use crate::{git, vmr};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn restore(
    working_dir: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> Result<()>
{
    // Find VMR root and route all requested paths before mutating any
    // repository
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let routed = vmr::route_paths(working_dir, &vmr_root, paths)?
        .into_iter()
        .collect::<Vec<_>>();

    // Restore paths in each child repository
    let mut failures = routed
        .par_iter()
        .filter_map(|(repo_path, repo_paths)| {
            repo_name(repo_path)
                .and_then(|repo_name| {
                    git::restore(
                        &repo_name, repo_path, repo_paths, worktree, staged
                    )
                })
                .transpose()
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

fn repo_name(repo_path: &Path) -> Result<String>
{
    Ok(repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("repository path has no valid UTF-8 file name")?
        .to_owned())
}
