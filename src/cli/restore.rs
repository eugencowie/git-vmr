use crate::config::vmr;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn restore(
    working_dir: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> Result<()>
{
    // Find VMR root and route all requested paths before mutating any
    // repository
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let routed = vmr::route_paths(working_dir, &vmr_root, paths)?;

    // Restore paths in each child repository
    for (repo_path, repo_paths) in routed
    {
        git_restore(&repo_path, &repo_paths, worktree, staged)?;
    }

    Ok(())
}

fn git_restore(
    repo_path: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> Result<()>
{
    // Build git restore command for the owning child repository
    let mut command = Command::new("git");
    command.arg("--no-optional-locks").arg("-C").arg(repo_path).arg("restore");

    if staged
    {
        command.arg("--staged");
    }

    if worktree
    {
        command.arg("--worktree");
    }

    // Run git restore with literal routed paths
    let output = command.arg("--").args(paths).output().with_context(|| {
        format!("failed to invoke git for '{}'", repo_path.display())
    })?;

    // Convert git failure into anyhow error
    if !output.status.success()
    {
        bail!(
            "git restore failed for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}
