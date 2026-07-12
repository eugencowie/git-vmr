use crate::render::{self, Rendered};
use crate::workspace::worktree_root::WorktreeRoots;
use crate::workspace::{Workspace, resolve_target};
use anyhow::Result;
use std::path::Path;

pub fn list(workspace: &Workspace) -> Result<Rendered>
{
    let repo_names = workspace
        .repos()
        .iter()
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();

    let groups = WorktreeRoots::new(workspace).list()?;

    Ok(render::worktree_list(groups, &repo_names).into())
}

pub fn add(
    workspace: &Workspace,
    working_dir: &Path,
    path: &Path,
    branch: Option<&str>,
    commit_ish: Option<&str>
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, path);
    render::outcomes(
        WorktreeRoots::new(workspace).add(&target, branch, commit_ish)?
    )
}

pub fn remove(
    workspace: &Workspace,
    working_dir: &Path,
    path: &Path,
    force: u8,
    delete: bool,
    force_delete: bool
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, path);
    let removal = WorktreeRoots::new(workspace).remove(
        &target,
        force,
        delete,
        force_delete
    )?;

    let rendered = render::outcomes(removal.outcomes)?;

    // Keep the per-repo successes if dissolving the root fails
    match removal.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(render::fail(rendered, format!("{error:#}")))
    }
}

pub fn move_worktree(
    workspace: &Workspace,
    working_dir: &Path,
    path: &Path,
    new_path: &Path,
    force: u8
) -> Result<Rendered>
{
    let source = resolve_target(working_dir, path);
    let destination = resolve_target(working_dir, new_path);

    let moved = WorktreeRoots::new(workspace).move_root(
        &source,
        &destination,
        force
    )?;

    let rendered = render::outcomes(moved.outcomes)?;

    // Keep the per-repo successes if dissolving the source root fails
    match moved.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(render::fail(rendered, format!("{error:#}")))
    }
}
