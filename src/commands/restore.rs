use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn restore(
    workspace: &Workspace,
    working_dir: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> Result<Rendered>
{
    // Restore routed paths in each owning child repository
    workspace.run_routed(
        working_dir,
        Scope::Paths(paths.to_vec()),
        |git, repo, repo_paths| git.restore(repo, repo_paths, worktree, staged)
    )
}
