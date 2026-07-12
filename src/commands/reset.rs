use crate::git::{ResetArgs, ResetMode};
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn reset(workspace: &Workspace, args: &ResetArgs) -> Result<Rendered>
{
    let mode = ResetMode::from_arg(
        args.soft, args.mixed, args.hard, args.merge, args.keep
    );

    // Reset each child repository
    workspace.run(|git, repo| git.reset(repo, mode, args.commit.as_deref()))
}
