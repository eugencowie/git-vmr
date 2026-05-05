use super::routing;
use crate::config::vmr;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn add(working_dir: &Path, paths: &[PathBuf]) -> Result<()>
{
    // Find VMR root and route requested paths
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let routed = routing::route_paths(working_dir, &vmr_root, paths)?;

    // Stage paths in each child repository
    for (repo_path, repo_paths) in routed
    {
        git_add(&repo_path, &repo_paths)?;
    }

    Ok(())
}

fn git_add(repo_path: &Path, paths: &[PathBuf]) -> Result<()>
{
    // Run git add in the owning child repository
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .arg("add")
        .arg("--")
        .args(paths)
        .output()
        .with_context(|| {
            format!("failed to invoke git for '{}'", repo_path.display())
        })?;

    // Convert git failure into anyhow error
    if !output.status.success()
    {
        bail!(
            "git add failed for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}
