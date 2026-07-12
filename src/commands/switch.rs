use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn switch(workspace: &Workspace, branch_name: &str) -> Result<Rendered>
{
    // Switch in each child repository
    workspace.run(|git, repo| git.switch(repo, branch_name))
}

pub fn create(workspace: &Workspace, branch_name: &str) -> Result<Rendered>
{
    // Create and switch in each child repository
    workspace.run(|git, repo| git.create(repo, branch_name))
}
