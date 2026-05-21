use crate::git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn merge(working_dir: &Path, commit_ish: &str) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Merge in each repository
    let results = repos
        .par_iter()
        .map(|repo| git::merge(&repo.name, &repo.path, commit_ish))
        .collect::<Vec<_>>();

    // Print results
    git::print_results(results)
}
