use crate::git::SwitchArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn switch(workspace: &Workspace, args: &SwitchArgs) -> Result<Rendered>
{
    match args.create
    {
        // Create and switch in each child repository
        true => workspace.run(|git, repo| git.create(repo, &args.branch_name)),

        // Switch in each child repository
        false => workspace.run(|git, repo| git.switch(repo, &args.branch_name))
    }
}
