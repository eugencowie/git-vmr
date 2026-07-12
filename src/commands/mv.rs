use crate::render::{self, Rendered};
use crate::workspace::{MvArgs, Workspace};
use anyhow::Result;
use std::path::Path;

pub fn mv(
    workspace: &Workspace,
    working_dir: &Path,
    args: &MvArgs
) -> Result<Rendered>
{
    let moved = workspace.mv(working_dir, args)?;
    render::outcomes_in_scope(moved.outcomes, moved.scope_repo_count)
}
