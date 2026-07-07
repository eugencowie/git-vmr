use crate::git::ResetMode;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn reset(
    workspace: &Workspace,
    mode: Option<ResetMode>,
    commit: Option<&str>
) -> Result<Rendered>
{
    // Reset each child repository
    workspace.run(|git, repo| git.reset(&repo.name, &repo.path, mode, commit))
}
