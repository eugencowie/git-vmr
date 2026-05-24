use crate::git::{git_path_output, stderr};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub fn mv(repo_path: &Path, source: &Path, destination: &Path) -> Result<()>
{
    let output =
        git_path_output(repo_path, ["mv", "--"], [source, destination])?;

    if !output.status.success()
    {
        bail!(
            "git mv failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}

pub fn mv_to_directory(
    repo_path: &Path,
    sources: &[PathBuf],
    destination: &Path
) -> Result<()>
{
    let paths = sources
        .iter()
        .map(PathBuf::as_path)
        .chain(std::iter::once(destination));
    let output = git_path_output(repo_path, ["mv", "--"], paths)?;

    if !output.status.success()
    {
        bail!(
            "git mv failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}

pub fn ensure_tracked(repo_path: &Path, path: &Path) -> Result<()>
{
    let output =
        git_path_output(repo_path, ["ls-files", "--error-unmatch", "--"], [
            path
        ])?;

    if !output.status.success()
    {
        bail!(
            "source path is not tracked in '{}': {}",
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(())
}
