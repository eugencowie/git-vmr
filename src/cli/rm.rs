use crate::cli::AggregateError;
use crate::{git, vmr};
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn rm(working_dir: &Path, paths: &[PathBuf], recursive: bool)
-> Result<()>
{
    // Find VMR root
    let vmr_root = vmr::find_vmr_root(working_dir)?;

    // Require explicit recursive intent for aggregate root removal
    if !recursive && targets_vmr_root(working_dir, &vmr_root, paths)
    {
        bail!("cannot remove VMR root without -r");
    }

    // Route requested paths before removing anything
    let routed = vmr::route_paths(working_dir, &vmr_root, paths)?
        .into_iter()
        .collect::<Vec<_>>();

    // Remove paths in each child repository
    let mut failures = routed
        .par_iter()
        .filter_map(|(repo_path, repo_paths)| {
            repo_name(repo_path)
                .and_then(|repo_name| {
                    git::rm(&repo_name, repo_path, repo_paths, recursive)
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

fn targets_vmr_root(
    working_dir: &Path,
    vmr_root: &Path,
    paths: &[PathBuf]
) -> bool
{
    // Detect root expansion before routing turns it into child repository paths
    paths
        .iter()
        .any(|path| vmr::resolve_path(working_dir, path).clean() == vmr_root)
}

fn repo_name(repo_path: &Path) -> Result<String>
{
    Ok(repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("repository path has no valid UTF-8 file name")?
        .to_owned())
}
