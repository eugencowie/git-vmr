use crate::workspace::Workspace;
use anyhow::Result;

pub fn pull(
    workspace: &Workspace,
    repository: Option<&str>,
    refspecs: &[String]
) -> Result<()>
{
    // Pull in each child repository
    workspace
        .run(|git, repo| git.pull(&repo.name, &repo.path, repository, refspecs))
}
