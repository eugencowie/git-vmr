use crate::git::RestoreArgs;
use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::Result;
use std::path::Path;

pub fn restore(
    workspace: &Workspace,
    working_dir: &Path,
    args: &RestoreArgs
) -> Result<Rendered>
{
    // Restore routed paths in each owning child repository
    workspace.run_routed(
        working_dir,
        Scope::Paths(args.paths.to_vec()),
        |git, repo, repo_paths| git.restore(repo, repo_paths, &args.options)
    )
}
