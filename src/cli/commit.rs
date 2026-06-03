use crate::cli::AggregateError;
use crate::git::{self, git_output};
use crate::vmr::{Repo, Vmr};
use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::path::Path;

pub fn commit(working_dir: &Path, message: &str) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = eligible_repos(&vmr)?;

    if repos.is_empty()
    {
        bail!("nothing to commit");
    }

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for result in repos
        .par_iter()
        .map(|repo| git::commit(&repo.name, &repo.path, message))
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

fn eligible_repos(vmr: &Vmr) -> Result<Vec<Repo>>
{
    let mut repos = Vec::new();

    for repo in vmr.repos()?
    {
        if has_staged_changes(&repo.path)?
        {
            repos.push(repo);
        }
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(repos)
}

fn has_staged_changes(repo_path: &Path) -> Result<bool>
{
    let output = git_output(repo_path, ["diff", "--cached", "--quiet"])
        .with_context(|| {
            format!(
                "failed to inspect staged changes for '{}'",
                repo_path.display()
            )
        })?;

    match output.status.code()
    {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => bail!(
            "git diff --cached --quiet failed for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}
