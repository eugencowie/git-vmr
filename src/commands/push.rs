use crate::git;
use crate::git::Git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn push(
    git: &Git,
    working_dir: &Path,
    repository: Option<&str>,
    refspecs: &[String]
) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Push in each repository
    let results = repos
        .par_iter()
        .map(|repo| git.push(&repo.name, &repo.path, repository, refspecs))
        .collect::<Vec<_>>();

    // Print results
    git::print_results(results)
}
