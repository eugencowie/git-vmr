use crate::workspace::Workspace;
use anyhow::Result;

pub fn rebase(workspace: &Workspace, upstream: &str) -> Result<()>
{
    // Rebase in each child repository
    workspace.run(|git, repo| git.rebase(&repo.name, &repo.path, upstream))
}
