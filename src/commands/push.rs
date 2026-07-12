use crate::git::PushArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn push(workspace: &Workspace, args: &PushArgs) -> Result<Rendered>
{
    // Push in each child repository
    workspace.run(|git, repo| git.push(repo, args))
}
