use crate::git::BranchArgs;
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn branch(workspace: &Workspace, args: &BranchArgs) -> Result<Rendered>
{
    match &args.branch_name
    {
        Some(branch_name) if args.delete || args.force_delete =>
            delete(workspace, branch_name, args.force || args.force_delete),
        Some(branch_name) => create(workspace, branch_name),
        None => branches(workspace)
    }
}

fn create(workspace: &Workspace, branch_name: &str) -> Result<Rendered>
{
    // Branch in each child repository
    workspace.run(|git, repo| git.branch(repo, branch_name))
}

fn delete(
    workspace: &Workspace,
    branch_name: &str,
    force: bool
) -> Result<Rendered>
{
    // Delete branch in each child repository
    workspace.run(|git, repo| git.delete_branch(repo, branch_name, force))
}

fn branches(workspace: &Workspace) -> Result<Rendered>
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
