use crate::cli::CliContext;
use crate::git::MergeArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: MergeArgs
) -> Result<Rendered>
{
    // Merge in each child repository
    workspace.run(|git, repo| git.merge(repo, &args))
}
