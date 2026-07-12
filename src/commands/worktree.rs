use crate::render::{self, Rendered};
use crate::workspace::{
    Workspace, WorktreeAddArgs, WorktreeCommand, WorktreeMoveArgs,
    WorktreeRemoveArgs, resolve_target
};
use anyhow::Result;
use std::path::Path;

pub fn worktree(
    workspace: &Workspace,
    working_dir: &Path,
    command: Option<WorktreeCommand>
) -> Result<Rendered>
{
    match command.unwrap_or(WorktreeCommand::List)
    {
        WorktreeCommand::Add(args) => add(workspace, working_dir, &args),
        WorktreeCommand::List => list(workspace),
        WorktreeCommand::Move(args) =>
            move_worktree(workspace, working_dir, &args),
        WorktreeCommand::Remove(args) => remove(workspace, working_dir, &args)
    }
}

fn list(workspace: &Workspace) -> Result<Rendered>
{
    let repo_names = workspace
        .repos()
        .iter()
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();

    let groups = workspace.worktree_roots().list()?;

    Ok(render::worktree_list(groups, &repo_names).into())
}

fn add(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeAddArgs
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, &args.path);
    render::outcomes(workspace.worktree_roots().add(
        &target,
        args.branch.as_deref(),
        args.commit_ish.as_deref()
    )?)
}

fn remove(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeRemoveArgs
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, &args.path);
    let removal = workspace.worktree_roots().remove(
        &target,
        args.force,
        args.delete,
        args.force_delete
    )?;

    let rendered = render::outcomes(removal.outcomes)?;

    // Keep the per-repo successes if dissolving the root fails
    match removal.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(render::fail(rendered, format!("{error:#}")))
    }
}

fn move_worktree(
    workspace: &Workspace,
    working_dir: &Path,
    args: &WorktreeMoveArgs
) -> Result<Rendered>
{
    let source = resolve_target(working_dir, &args.path);
    let destination = resolve_target(working_dir, &args.new_path);

    let moved = workspace.worktree_roots().move_root(
        &source,
        &destination,
        args.force
    )?;

    let rendered = render::outcomes(moved.outcomes)?;

    // Keep the per-repo successes if dissolving the source root fails
    match moved.dissolved
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(render::fail(rendered, format!("{error:#}")))
    }
}
