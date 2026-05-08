use crate::cli::AggregateError;
use crate::git;
use crate::vmr::{self, Vmr};
use anyhow::{Result, bail};
use path_clean::PathClean;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn rm(working_dir: &Path, paths: &[PathBuf], recursive: bool)
-> Result<()>
{
    // Find VMR root
    let vmr = Vmr::find(working_dir)?;

    // Require explicit recursive intent for aggregate root removal
    if !recursive && targets_vmr_root(working_dir, vmr.root(), paths)
    {
        bail!("cannot remove VMR root without -r");
    }

    // Route requested paths before removing anything
    let routed =
        vmr.route_paths(working_dir, paths)?.into_iter().collect::<Vec<_>>();

    // Remove paths in each child repository
    let mut failures = routed
        .par_iter()
        .filter_map(|(repo_path, repo_paths)| {
            git::rm(&repo_path.name, &repo_path.path, repo_paths, recursive)
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
