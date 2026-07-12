use crate::git::{BranchAction, BranchArgs};
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn branch(workspace: &Workspace, args: &BranchArgs) -> Result<Rendered>
{
    match args.action()
    {
        BranchAction::List => branches(workspace),

        // Branch in each child repository
        BranchAction::Create(branch_name) =>
            workspace.run(|git, repo| git.branch(repo, branch_name)),

        // Delete branch in each child repository
        BranchAction::Delete { branch_name, force } => workspace
            .run(|git, repo| git.delete_branch(repo, branch_name, force))
    }
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
