use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn merge(workspace: &Workspace, commit_ish: &str) -> Result<Rendered>
{
    // Merge in each child repository
    workspace.run(|git, repo| git.merge(&repo.name, &repo.path, commit_ish))
}
