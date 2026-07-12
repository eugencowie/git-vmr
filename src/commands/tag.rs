use crate::git::{TagAction, TagArgs};
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn tag(workspace: &Workspace, args: &TagArgs) -> Result<Rendered>
{
    match args.action()
    {
        TagAction::List => tags(workspace),

        // Create tag in each child repository
        TagAction::Create(tag_name) =>
            workspace.run(|git, repo| git.tag(repo, tag_name)),

        // Delete tag in each child repository
        TagAction::Delete(tag_name) =>
            workspace.run(|git, repo| git.delete_tag(repo, tag_name)),
    }
}

fn tags(workspace: &Workspace) -> Result<Rendered>
{
    // Collect tag information from child repositories, in repo order
    let tags = workspace
        .map(|git, repo| git.tags(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render results
    Ok(render::tags(&tags).into())
}
