use crate::vmr::Vmr;
use crate::{cli, git};
use anyhow::{Result, bail};
use rayon::prelude::*;
use std::path::Path;

pub fn commit(working_dir: &Path, message: &str) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Filter repositories without staged changes
    let dirty_repos = repos
        .par_iter()
        .filter_map(|repo| match repo.is_dirty()
        {
            Ok(true) => Some(Ok(repo)),
            Ok(false) => None,
            Err(error) => Some(Err(error))
        })
        .collect::<Result<Vec<_>>>()?;

    // Check if there are no dirty repositories
    if dirty_repos.is_empty()
    {
        bail!("nothing to commit, working tree clean");
    }

    // Commit in each dirty repository
    let results = dirty_repos
        .par_iter()
        .map(|repo| git::commit(&repo.name, &repo.path, message))
        .collect::<Vec<_>>();

    // Print results
    cli::print_results(results)
}
