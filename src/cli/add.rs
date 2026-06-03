use crate::cli::print_results;
use crate::git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn add(working_dir: &Path, paths: &[PathBuf]) -> Result<()>
{
    // Find VMR root and route requested paths
    let vmr = Vmr::find(working_dir)?;
    let routed =
        vmr.route_paths(working_dir, paths)?.into_iter().collect::<Vec<_>>();

    // Stage paths in each child repository
    let results = routed
        .par_iter()
        .map(|(repo_path, repo_paths)| {
            git::add(&repo_path.name, &repo_path.path, repo_paths)
        })
        .collect::<Vec<_>>();

    print_results(results)
}
