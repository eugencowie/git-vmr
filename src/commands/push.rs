use crate::workspace::Workspace;
use anyhow::Result;

pub fn push(
    workspace: &Workspace,
    repository: Option<&str>,
    refspecs: &[String]
) -> Result<()>
{
    // Push in each child repository
    workspace
        .run(|git, repo| git.push(&repo.name, &repo.path, repository, refspecs))
}
