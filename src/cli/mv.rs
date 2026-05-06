use crate::config::vmr;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn mv(working_dir: &Path, source: &Path, destination: &Path) -> Result<()>
{
    // Find VMR root and route both operands before moving anything
    let vmr_root = vmr::find_vmr_root(working_dir)?;
    let source = vmr::route_single_path(working_dir, &vmr_root, source)?;
    let destination =
        vmr::route_single_path(working_dir, &vmr_root, destination)?;

    // Delegate same-repository moves to Git and synthesize cross-repository
    // moves
    if source.0 == destination.0
    {
        git_mv(&source.0, &source.1, &destination.1)
    }
    else
    {
        mv_between_repos(source, destination)
    }
}

fn git_mv(repo_path: &Path, source: &Path, destination: &Path) -> Result<()>
{
    // Run git mv in the owning child repository
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .arg("mv")
        .arg("--")
        .arg(source)
        .arg(destination)
        .output()
        .with_context(|| {
            format!("failed to invoke git for '{}'", repo_path.display())
        })?;

    // Convert git failure into anyhow error
    if !output.status.success()
    {
        bail!(
            "git mv failed for '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

fn mv_between_repos(
    source: (PathBuf, PathBuf),
    destination: (PathBuf, PathBuf)
) -> Result<()>
{
    let (source_repo, source_relative) = source;
    let (destination_repo, destination_relative) = destination;

    // Validate source tracking before changing the filesystem
    ensure_tracked(&source_repo, &source_relative)?;

    // Resolve destination-directory semantics before moving
    let source_path = source_repo.join(&source_relative);
    let final_destination_relative = final_destination_path(
        &source_relative,
        &destination_repo,
        &destination_relative
    )?;
    let destination_path = destination_repo.join(&final_destination_relative);

    // Move the worktree path across child repositories
    fs::rename(&source_path, &destination_path).with_context(|| {
        format!(
            "failed to move '{}' to '{}'",
            source_path.display(),
            destination_path.display()
        )
    })?;

    // Stage the source deletion and destination addition in their repositories
    git_add(&source_repo, &source_relative).with_context(|| {
        format!(
            "failed to stage source deletion in '{}'",
            source_repo.display()
        )
    })?;
    git_add(&destination_repo, &final_destination_relative).with_context(
        || {
            format!(
                "failed to stage destination addition in '{}'",
                destination_repo.display()
            )
        }
    )?;

    Ok(())
}

fn ensure_tracked(repo_path: &Path, path: &Path) -> Result<()>
{
    // Ask Git whether the source path is tracked
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .arg("ls-files")
        .arg("--error-unmatch")
        .arg("--")
        .arg(path)
        .output()
        .with_context(|| {
            format!("failed to invoke git for '{}'", repo_path.display())
        })?;

    // Convert untracked source into a preflight failure
    if !output.status.success()
    {
        bail!(
            "source path is not tracked in '{}': {}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

fn final_destination_path(
    source_relative: &Path,
    destination_repo: &Path,
    destination_relative: &Path
) -> Result<PathBuf>
{
    let destination_path = destination_repo.join(destination_relative);

    // Treat an existing destination directory like git mv does
    if destination_path.is_dir()
    {
        let source_name = source_relative
            .file_name()
            .context("source path does not have a file name")?;
        return Ok(destination_relative.join(source_name));
    }

    // Reject conflicts and missing parents before moving the source
    if destination_path.exists()
    {
        bail!("destination '{}' already exists", destination_path.display());
    }

    let parent = destination_path
        .parent()
        .context("destination path does not have a parent")?;
    if !parent.is_dir()
    {
        bail!("destination parent '{}' does not exist", parent.display());
    }

    Ok(destination_relative.to_path_buf())
}

fn git_add(repo_path: &Path, path: &Path) -> Result<()>
{
    // Run git add in the owning child repository
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .arg("add")
        .arg("--")
        .arg(path)
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
