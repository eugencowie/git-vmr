use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn push(
    workspace: &Workspace,
    repository: Option<&str>,
    refspecs: &[String]
) -> Result<Rendered>
{
    // Push in each child repository
    workspace
        .run(|git, repo| git.push(&repo.name, &repo.path, repository, refspecs))
}
