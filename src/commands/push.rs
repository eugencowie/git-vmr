use crate::cli::CliContext;
use crate::git::PushArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: PushArgs
) -> Result<Rendered>
{
    // Push in each child repository
    workspace.run(|git, repo| git.push(repo, &args))
}
