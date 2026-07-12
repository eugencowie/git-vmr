use crate::git;
use crate::git::Git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn rebase(git: &Git, working_dir: &Path, upstream: &str) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Rebase in each repository
    let results = repos
        .par_iter()
        .map(|repo| git.rebase(&repo.name, &repo.path, upstream))
        .collect::<Vec<_>>();

    // Print results
    git::print_results(results)
}
