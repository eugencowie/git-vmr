use crate::vmr::Vmr;
use crate::{cli, git};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn switch(working_dir: &Path, branch_name: &str) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Switch in each repository
    let results = repos
        .par_iter()
        .map(|repo| git::switch(&repo.name, &repo.path, branch_name))
        .collect::<Vec<_>>();

    // Print results
    cli::print_results(results)
}

pub fn create(working_dir: &Path, branch_name: &str) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Create and switch in each repository
    let results = repos
        .par_iter()
        .map(|repo| git::create(&repo.name, &repo.path, branch_name))
        .collect::<Vec<_>>();

    // Print results
    cli::print_results(results)
}
