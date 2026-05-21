use crate::git;
use crate::vmr::Vmr;
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
    if !recursive && targets_vmr_root(working_dir, &vmr.path, paths)
    {
        bail!("error: cannot remove VMR root without -r");
    }

    // Route requested paths before removing anything
    let routed =
        vmr.route_paths(working_dir, paths)?.into_iter().collect::<Vec<_>>();

    // Remove paths in each child repository
    let results = routed
        .par_iter()
        .map(|(repo_path, repo_paths)| {
            git::rm(&repo_path.name, &repo_path.path, repo_paths, recursive)
        })
        .collect::<Vec<_>>();

    git::print_results(results)
}

fn targets_vmr_root(
    working_dir: &Path,
    vmr_root: &Path,
    paths: &[PathBuf]
) -> bool
{
    // Detect root expansion before routing turns it into child repository paths
    paths.iter().any(|path| resolve_path(working_dir, path).clean() == vmr_root)
}

fn resolve_path(working_dir: &Path, path: &Path) -> PathBuf
{
    // Interpret relative paths against effective working directory
    if path.is_absolute() { path.to_path_buf() } else { working_dir.join(path) }
}
