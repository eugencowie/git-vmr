use crate::cli::AggregateError;
use crate::git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn rebase(working_dir: &Path, upstream: &str) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;

    let mut failures = repos
        .par_iter()
        .filter_map(|repo| {
            git::rebase(&repo.name, &repo.path, upstream).transpose()
        })
        .collect::<Result<Vec<_>>>()?;

    failures.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

    if !failures.is_empty()
    {
        return Err(AggregateError::new(
            failures
                .into_iter()
                .map(|failure| {
                    anyhow::anyhow!(
                        "{} ({})",
                        failure.message,
                        failure.repo_name
                    )
                })
                .collect()
        )
        .into());
    }

    Ok(())
}
