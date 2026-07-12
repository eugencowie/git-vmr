use crate::git::SwitchArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn switch(workspace: &Workspace, args: &SwitchArgs) -> Result<Rendered>
{
    // Switch in each child repository, creating the branch when asked
    workspace.run(|git, repo| git.switch(repo, args))
}
