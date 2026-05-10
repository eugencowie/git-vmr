use crate::cli::AggregateError;
use crate::git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn add(working_dir: &Path, paths: &[PathBuf]) -> Result<()>
{
    // Find VMR root and route requested paths
    let vmr = Vmr::find(working_dir)?;
    let routed =
        vmr.route_paths(working_dir, paths)?.into_iter().collect::<Vec<_>>();

    // Stage paths in each child repository
    let mut failures = routed
        .par_iter()
        .filter_map(|(repo_path, repo_paths)| {
            git::add(&repo_path.name, &repo_path.path, repo_paths)
                .map(|outcome| outcome.into_failure())
                .transpose()
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
