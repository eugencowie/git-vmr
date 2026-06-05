use crate::vmr::Vmr;
use crate::{cli, git};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn push(
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
        .map(|repo| git::push(&repo.name, &repo.path, repository, refspecs))
        .collect::<Vec<_>>();

    // Print results
    cli::print_results(results)
}
