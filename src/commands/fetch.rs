use crate::git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn fetch(
    working_dir: &Path,
    repository: Option<&str>,
    refspecs: &[String]
) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Fetch in each repository
    let results = repos
        .par_iter()
        .map(|repo| git::fetch(&repo.name, &repo.path, repository, refspecs))
        .collect::<Vec<_>>();

    // Print results
    git::print_results(results)
}
