use crate::git;
use crate::vmr::{Vmr, resolve_path};
use anyhow::{Context, Result};
use path_clean::PathClean;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

pub fn add(
    working_dir: &Path,
    path: &Path,
    commit_ish: Option<&str>
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let target = resolve_path(working_dir, path).clean();
    let branch = match commit_ish
    {
        Some(_) => None,
        None => Some(
            target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .context("fatal: worktree target path must have a basename")?
        )
    };

    fs::create_dir_all(&target).with_context(|| {
        format!(
            "fatal: failed to create worktree target '{}'",
            target.display()
        )
    })?;
    fs::write(target.join(".gitvmr"), "").with_context(|| {
        format!("fatal: failed to create VMR marker in '{}'", target.display())
    })?;

    let results = repos
        .par_iter()
        .map(|repo| {
            git::worktree_add(
                &repo.name,
                &repo.path,
                &target.join(&repo.name),
                branch.as_deref(),
                commit_ish
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)
}
