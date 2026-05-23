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
    branch: Option<&str>,
    commit_ish: Option<&str>
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let target = resolve_path(working_dir, path).clean();
    let branch = match (branch, commit_ish)
    {
        (Some(branch), _) => Some(branch.to_owned()),
        (None, Some(_)) => None,
        (None, None) => Some(
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

pub fn remove(working_dir: &Path, path: &Path, force: u8) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let target = resolve_path(working_dir, path).clean();

    let results = repos
        .par_iter()
        .map(|repo| {
            git::worktree_remove(
                &repo.name,
                &repo.path,
                &target.join(&repo.name),
                force
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)?;

    let marker = target.join(".gitvmr");
    if marker.exists()
    {
        if marker.is_dir()
        {
            fs::remove_dir(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
        else
        {
            fs::remove_file(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
    }

    match fs::remove_dir(&target)
    {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty =>
            Ok(()),
        Err(error) => Err(error).with_context(|| {
            format!(
                "fatal: failed to remove empty worktree directory '{}'",
                target.display()
            )
        })
    }
}

pub fn move_worktree(
    working_dir: &Path,
    path: &Path,
    new_path: &Path,
    force: u8
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let source = resolve_path(working_dir, path).clean();
    let destination = resolve_path(working_dir, new_path).clean();

    fs::create_dir_all(&destination).with_context(|| {
        format!(
            "fatal: failed to create worktree target '{}'",
            destination.display()
        )
    })?;
    fs::write(destination.join(".gitvmr"), "").with_context(|| {
        format!(
            "fatal: failed to create VMR marker in '{}'",
            destination.display()
        )
    })?;

    let results = repos
        .par_iter()
        .map(|repo| {
            git::worktree_move(
                &repo.name,
                &repo.path,
                &source.join(&repo.name),
                &destination.join(&repo.name),
                force
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)?;

    let marker = source.join(".gitvmr");
    if marker.exists()
    {
        if marker.is_dir()
        {
            fs::remove_dir(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
        else
        {
            fs::remove_file(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
    }

    match fs::remove_dir(&source)
    {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty =>
            Ok(()),
        Err(error) => Err(error).with_context(|| {
            format!(
                "fatal: failed to remove empty worktree directory '{}'",
                source.display()
            )
        })
    }
}
