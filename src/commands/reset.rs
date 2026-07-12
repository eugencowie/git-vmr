use crate::git::ResetArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn reset(workspace: &Workspace, args: &ResetArgs) -> Result<Rendered>
{
    let mode = args.mode();

    // Reset each child repository
    workspace.run(|git, repo| git.reset(repo, mode, args.commit.as_deref()))
}
