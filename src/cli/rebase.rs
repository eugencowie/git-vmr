use crate::cli::AggregateError;
use crate::config::vmr;
use crate::git;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn rebase(working_dir: &Path, upstream: &str) -> Result<()>
{
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let repos = vmr::eligible_repos(&vmr_root)?;

    let mut failures = repos
        .par_iter()
        .filter_map(|(repo_name, repo_path)| {
            git::rebase(repo_name, repo_path, upstream).transpose()
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
