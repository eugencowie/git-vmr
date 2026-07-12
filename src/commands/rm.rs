use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub fn rm(
    workspace: &Workspace,
    working_dir: &Path,
    paths: &[PathBuf],
    recursive: bool,
    force: bool,
    dry_run: bool,
    cached: bool
) -> Result<Rendered>
{
    // Require explicit recursive intent for aggregate path removal
    if !recursive
        && paths.iter().any(|path| workspace.is_aggregate(working_dir, path))
    {
        bail!("error: cannot remove VMR root without -r");
    }

    // Remove routed paths in each owning child repository
    workspace.run_routed(
        working_dir,
        Scope::Paths(paths.to_vec()),
        |git, repo, repo_paths| {
            git.rm(repo, repo_paths, recursive, force, dry_run, cached)
        }
    )
}
