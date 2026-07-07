use crate::workspace::Workspace;
use anyhow::Result;

pub fn switch(workspace: &Workspace, branch_name: &str) -> Result<()>
{
    // Switch in each child repository
    workspace.run(|git, repo| git.switch(&repo.name, &repo.path, branch_name))
}

pub fn create(workspace: &Workspace, branch_name: &str) -> Result<()>
{
    // Create and switch in each child repository
    workspace.run(|git, repo| git.create(&repo.name, &repo.path, branch_name))
}
