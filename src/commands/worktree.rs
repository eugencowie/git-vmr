use crate::git;
use crate::git::Git;
use crate::render::{self, Rendered, WorktreeRootEntry};
use crate::vmr::Vmr;
use crate::workspace::{Repo, Workspace, resolve_target};
use anyhow::{Context, Result};
use path_clean::PathClean;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn list(workspace: &Workspace) -> Result<Rendered>
{
    let repo_names = workspace
        .repos()
        .iter()
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();

    let results = workspace.map(|git, repo| {
        git.worktree_list(&repo.name, &repo.path)
            .map(|entries| (repo.name.clone(), entries))
    })?;

    let mut groups: BTreeMap<PathBuf, Vec<WorktreeRootEntry>> = BTreeMap::new();
    for (repo_name, entries) in results
    {
        for entry in entries
        {
            if let Some(root) =
                worktree_root(workspace.root(), &repo_name, &entry.path)
            {
                groups.entry(root).or_default().push(WorktreeRootEntry {
                    repo: repo_name.clone(),
                    head: entry.head,
                    state: entry.state
                });
            }
        }
    }

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
    let mode = match (branch, commit_ish)
    {
        (Some(branch), commit_ish) => WorktreeAddMode::NewBranch {
            branch: branch.to_owned(),
            commit_ish: commit_ish.map(str::to_owned)
        },
        (None, Some(commit_ish)) =>
            WorktreeAddMode::CommitIsh(commit_ish.to_owned()),
        (None, None) => WorktreeAddMode::InferredBranch(
            target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .context("fatal: worktree target path must have a basename")?
        )
    };

    Vmr::create_worktree_root(&target)?;

    workspace.run(|git, repo| {
        let (branch, commit_ish) = match &mode
        {
            WorktreeAddMode::NewBranch { branch, commit_ish } =>
                (Some(branch.as_str()), commit_ish.as_deref()),
            WorktreeAddMode::CommitIsh(commit_ish) =>
                (None, Some(commit_ish.as_str())),
            WorktreeAddMode::InferredBranch(branch)
                if git.branch_exists(&repo.path, branch)? =>
                (None, Some(branch.as_str())),
            WorktreeAddMode::InferredBranch(branch) =>
                (Some(branch.as_str()), None),
        };

        git.worktree_add(
            &repo.name,
            &repo.path,
            &target.join(&repo.name),
            branch,
            commit_ish
        )
    })
}

enum WorktreeAddMode
{
    NewBranch
    {
        branch: String,
        commit_ish: Option<String>
    },
    CommitIsh(String),
    InferredBranch(String)
}

fn worktree_root(
    vmr_root: &Path,
    repo_name: &str,
    worktree_path: &Path
) -> Option<PathBuf>
{
    let main_child = vmr_root.join(repo_name);
    if worktree_path == main_child
    {
        return Some(vmr_root.to_owned());
    }

    if worktree_path.file_name()? != repo_name
    {
        return None;
    }

    let root = worktree_path.parent()?.to_owned();
    if Vmr::is_root(&root) { Some(root) } else { None }
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
    let git = workspace.git();
    let target = resolve_target(working_dir, path);

    let removals = workspace.map(|git, repo| {
        // Wrap per-repo errors so map attempts every repository.
        Ok(removal_outcome(git, repo, &target, force, delete, force_delete))
    })?;

    let mut all_child_removals_succeeded = true;
    let mut results = Vec::new();
    let mut deletion_targets = Vec::new();
    for removal in removals
    {
        match removal
        {
            Ok(RemovalOutcome { repo_name, repo_path, branch, result }) =>
            {
                if !matches!(result, Ok(git::RepoOutcome::Success(_)))
                {
                    all_child_removals_succeeded = false;
                }
                else if let Some(branch) = branch
                {
                    deletion_targets.push((repo_name, repo_path, branch));
                }

                results.push(result);
            }
            Err(error) =>
            {
                all_child_removals_succeeded = false;
                results.push(Err(error));
            }
        }
    }

    if delete || force_delete
    {
        let branch_deletions = deletion_targets
            .par_iter()
            .map(|(repo_name, repo_path, branch)| {
                git.delete_branch(repo_name, repo_path, branch, force_delete)
            })
            .collect::<Vec<_>>();
        results.extend(branch_deletions);
    }

    let rendered = render::outcomes(results)?;

    if all_child_removals_succeeded
        && let Err(error) = Vmr::remove_worktree_root(&target)
    {
        return Err(render::fail(rendered, format!("{error:#}")));
    }

    Ok(rendered)
}

struct RemovalOutcome
{
    repo_name: String,
    repo_path: PathBuf,
    branch: Option<String>,
    result: git::GitCommandResult
}

fn removal_outcome(
    git: &Git,
    repo: &Repo,
    target: &Path,
    force: u8,
    delete: bool,
    force_delete: bool
) -> Result<RemovalOutcome>
{
    let child_target = target.join(&repo.name).clean();
    let branch = if delete || force_delete
    {
        child_worktree_branch(git, &repo.name, &repo.path, &child_target)?
    }
    else
    {
        None
    };
    let result =
        git.worktree_remove(&repo.name, &repo.path, &child_target, force);

    Ok(RemovalOutcome {
        repo_name: repo.name.clone(),
        repo_path: repo.path.clone(),
        branch,
        result
    })
}

fn child_worktree_branch(
    git: &Git,
    repo_name: &str,
    repo_path: &Path,
    child_target: &Path
) -> Result<Option<String>>
{
    let entries = git.worktree_list(repo_name, repo_path)?;
    let child_target = child_target.clean();

    Ok(entries
        .into_iter()
        .find(|entry| entry.path.clean() == child_target)
        .and_then(|entry| match entry.state
        {
            git::ChildWorktreeState::Branch(branch) => Some(branch),
            git::ChildWorktreeState::Detached => None
        }))
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

    Vmr::create_worktree_root(&destination)?;

    let rendered = workspace.run(|git, repo| {
        git.worktree_move(
            &repo.name,
            &repo.path,
            &source.join(&repo.name),
            &destination.join(&repo.name),
            force
        )
    })?;

    // Keep the per-repo successes if dissolving the source root fails
    match Vmr::remove_worktree_root(&source)
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(render::fail(rendered, format!("{error:#}")))
    }
}
