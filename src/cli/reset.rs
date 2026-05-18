use crate::cli;
use crate::git::{self, ResetMode};
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn reset(
    working_dir: &Path,
    mode: Option<ResetMode>,
    commit: Option<&str>
) -> Result<()>
{
    // Find virtual monorepo
    let vmr = Vmr::find(working_dir)?;

    // Get list of repositories
    let repos = vmr.repos()?;

    // Reset each repository
    let results = repos
        .par_iter()
        .map(|repo| git::reset(&repo.name, &repo.path, mode, commit))
        .collect::<Vec<_>>();

    // Print results
    cli::print_results(results)
}
