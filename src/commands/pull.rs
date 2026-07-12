use crate::cli::CliContext;
use crate::git::PullArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: PullArgs
) -> Result<Rendered>
{
    // Pull in each child repository
    workspace.run(|git, repo| git.pull(repo, &args))
}
