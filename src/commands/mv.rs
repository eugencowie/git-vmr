use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn mv(
    workspace: &Workspace,
    working_dir: &Path,
    sources: &[PathBuf],
    destination: &Path
) -> Result<Rendered>
{
    let moved = workspace.mv(working_dir, sources, destination)?;
    render::outcomes_in_scope(moved.outcomes, moved.scope_repo_count)
}
