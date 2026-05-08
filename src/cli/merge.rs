use crate::cli::AggregateError;
use crate::git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub fn merge(working_dir: &Path, commit_ish: &str) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for result in repos
        .par_iter()
        .map(|repo| git::merge(&repo.name, &repo.path, commit_ish))
        .collect::<Result<Vec<_>>>()?
    {
        match result
        {
            Ok(success) => successes.push(success),
            Err(failure) => failures.push(failure)
        }
    }

    successes.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));
    failures.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

    for success in successes
    {
        println!("{} ({})", success.stdout, success.repo_name);
    }

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
