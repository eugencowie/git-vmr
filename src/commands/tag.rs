use crate::git::TagArgs;
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn tag(workspace: &Workspace, args: &TagArgs) -> Result<Rendered>
{
    match (&args.tag_name, args.delete)
    {
        (Some(tag_name), true) => delete(workspace, tag_name),
        (Some(tag_name), false) => create(workspace, tag_name),
        (None, false) => tags(workspace),
        _ => unreachable!()
    }
}

fn create(workspace: &Workspace, tag_name: &str) -> Result<Rendered>
{
    // Create tag in each child repository
    workspace.run(|git, repo| git.tag(repo, tag_name))
}

fn delete(workspace: &Workspace, tag_name: &str) -> Result<Rendered>
{
    // Delete tag in each child repository
    workspace.run(|git, repo| git.delete_tag(repo, tag_name))
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
