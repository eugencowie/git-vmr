use crate::config::vmr;
use anyhow::{Context, Result, bail};
use path_clean::PathClean;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn rm(working_dir: &Path, paths: &[PathBuf], recursive: bool)
-> Result<()>
{
    // Find VMR root
    let vmr_root = vmr::find_vmr_root(working_dir)?;

    // Require explicit recursive intent for aggregate root removal
    if !recursive && targets_vmr_root(working_dir, &vmr_root, paths)
    {
        bail!("cannot remove VMR root without -r");
    }

    // Route requested paths before removing anything
    let routed = vmr::route_paths(working_dir, &vmr_root, paths)?;

    // Remove paths in each child repository
    for (repo_path, repo_paths) in routed
    {
        git_rm(&repo_path, &repo_paths, recursive)?;
    }

    Ok(())
}

fn targets_vmr_root(
    working_dir: &Path,
    vmr_root: &Path,
    paths: &[PathBuf]
) -> bool
{
    // Detect root expansion before routing turns it into child repository paths
    paths
        .iter()
        .any(|path| vmr::resolve_path(working_dir, path).clean() == vmr_root)
}

fn git_rm(repo_path: &Path, paths: &[PathBuf], recursive: bool) -> Result<()>
{
    // Build git rm command for the owning child repository
    let mut command = Command::new("git");
    command.arg("--no-optional-locks").arg("-C").arg(repo_path).arg("rm");

    if recursive
    {
        command.arg("-r");
    }

    // Run git rm with literal routed paths
    let output = command.arg("--").args(paths).output().with_context(|| {
        format!("failed to invoke git for '{}'", repo_path.display())
    })?;

    // Convert git failure into anyhow error
    if !output.status.success()
    {
        bail!(
            "git rm failed for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}
