use crate::cli::CliContext;
use crate::git::RebaseArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: RebaseArgs
) -> Result<Rendered>
{
    // Rebase in each child repository
    workspace.run(|git, repo| git.rebase(repo, &args))
}
