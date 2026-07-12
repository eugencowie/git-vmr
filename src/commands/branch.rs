use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn branch(workspace: &Workspace, branch_name: &str) -> Result<Rendered>
{
    // Branch in each child repository
    workspace.run(|git, repo| git.branch(&repo.name, &repo.path, branch_name))
}

pub fn delete(
    workspace: &Workspace,
    branch_name: &str,
    force: bool
) -> Result<Rendered>
{
    // Delete branch in each child repository
    workspace.run(|git, repo| {
        git.delete_branch(&repo.name, &repo.path, branch_name, force)
    })
}

pub fn branches(workspace: &Workspace) -> Result<Rendered>
{
    // Collect branch information from child repositories, in repo order
    let branches = workspace
        .map(|git, repo| git.branches(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render results
    Ok(render::branches(&branches).into())
}
