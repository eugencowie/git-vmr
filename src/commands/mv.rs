use crate::cli::CliContext;
use crate::render::{self, Rendered};
use crate::workspace::{MvArgs, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: MvArgs
) -> Result<Rendered>
{
    let moved = workspace.mv(&context.working_dir, &args)?;
    render::outcomes_in_scope(moved.outcomes, moved.scope_repo_count)
}
