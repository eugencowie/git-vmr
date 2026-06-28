use crate::git;
use crate::vmr::{Vmr, resolve_path};
use anyhow::{Context, Result};
use path_clean::PathClean;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn list(working_dir: &Path) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let repo_names =
        repos.iter().map(|repo| repo.name.clone()).collect::<Vec<_>>();

    let results = repos
        .par_iter()
        .map(|repo| {
            git::worktree_list(&repo.name, &repo.path)
                .map(|entries| (repo.name.clone(), entries))
        })
        .collect::<Vec<_>>();

    let mut groups: BTreeMap<PathBuf, Vec<AggregateEntry>> = BTreeMap::new();
    for result in results
    {
        let (repo_name, entries) = result?;

        for entry in entries
        {
            if let Some(root) =
                aggregate_root(&vmr.path, &repo_name, &entry.path)
            {
                groups.entry(root).or_default().push(AggregateEntry {
                    repo: repo_name.clone(),
                    head: entry.head,
                    state: entry.state
                });
            }
        }
    }

    for (root, entries) in groups
    {
        render_group(&root, entries, &repo_names);
    }

    Ok(())
}

pub fn add(
    working_dir: &Path,
    path: &Path,
    branch: Option<&str>,
    commit_ish: Option<&str>
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let target = resolve_path(working_dir, path).clean();
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

    fs::create_dir_all(&target).with_context(|| {
        format!(
            "fatal: failed to create worktree target '{}'",
            target.display()
        )
    })?;
    fs::write(target.join(".gitvmr"), "").with_context(|| {
        format!("fatal: failed to create VMR marker in '{}'", target.display())
    })?;

    let results = repos
        .par_iter()
        .map(|repo| {
            let (branch, commit_ish) = match &mode
            {
                WorktreeAddMode::NewBranch { branch, commit_ish } =>
                    (Some(branch.as_str()), commit_ish.as_deref()),
                WorktreeAddMode::CommitIsh(commit_ish) =>
                    (None, Some(commit_ish.as_str())),
                WorktreeAddMode::InferredBranch(branch)
                    if git::branch_exists(&repo.path, branch)? =>
                    (None, Some(branch.as_str())),
                WorktreeAddMode::InferredBranch(branch) =>
                    (Some(branch.as_str()), None),
            };

            git::worktree_add(
                &repo.name,
                &repo.path,
                &target.join(&repo.name),
                branch,
                commit_ish
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)
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

fn aggregate_root(
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
    if root.join(".gitvmr").exists() { Some(root) } else { None }
}

#[derive(Clone, Eq, PartialEq)]
struct AggregateEntry
{
    repo: String,
    head: String,
    state: git::ChildWorktreeState
}

fn render_group(
    root: &Path,
    mut entries: Vec<AggregateEntry>,
    repo_names: &[String]
)
{
    entries.sort_by(|a, b| {
        (state_sort_key(&a.state), short_head(&a.head), &a.repo).cmp(&(
            state_sort_key(&b.state),
            short_head(&b.head),
            &b.repo
        ))
    });

    let mut state_groups: BTreeMap<RenderedState, Vec<String>> =
        BTreeMap::new();

    for entry in entries
    {
        state_groups
            .entry(RenderedState::from_entry(&entry))
            .or_default()
            .push(entry.repo);
    }

    let mut lines = Vec::new();
    for (state, mut repos) in state_groups
    {
        repos.sort();
        let suffix = if repos == repo_names
        {
            String::new()
        }
        else
        {
            format!(" ({})", repos.join(", "))
        };

        lines.push(format!("{}{}", state.render(), suffix));
    }

    if lines.len() == 1
    {
        println!("{} {}", git::git_style_path(root), lines[0]);
    }
    else
    {
        println!("{}", git::git_style_path(root));
        for line in lines
        {
            println!("  {line}");
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum RenderedState
{
    Branch(String),
    Detached(String)
}

impl RenderedState
{
    fn from_entry(entry: &AggregateEntry) -> Self
    {
        match &entry.state
        {
            git::ChildWorktreeState::Branch(branch) =>
                Self::Branch(branch.clone()),
            git::ChildWorktreeState::Detached =>
                Self::Detached(short_head(&entry.head).to_owned()),
        }
    }

    fn render(&self) -> String
    {
        match self
        {
            Self::Branch(branch) => format!("[{branch}]"),
            Self::Detached(head) => format!("{head} (detached HEAD)")
        }
    }
}

fn short_head(head: &str) -> &str
{
    head.get(..8).unwrap_or(head)
}

fn state_sort_key(state: &git::ChildWorktreeState) -> (&str, &str)
{
    match state
    {
        git::ChildWorktreeState::Branch(branch) => ("branch", branch),
        git::ChildWorktreeState::Detached => ("detached", "")
    }
}

pub fn remove(
    working_dir: &Path,
    path: &Path,
    force: u8,
    delete: bool,
    force_delete: bool
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let target = resolve_path(working_dir, path).clean();

    let removals = repos
        .par_iter()
        .map(|repo| {
            let child_target = target.join(&repo.name).clean();
            let branch = if delete || force_delete
            {
                child_worktree_branch(&repo.name, &repo.path, &child_target)?
            }
            else
            {
                None
            };
            let result = git::worktree_remove(
                &repo.name,
                &repo.path,
                &child_target,
                force
            );

            Ok(RemovalOutcome {
                repo_name: repo.name.clone(),
                repo_path: repo.path.clone(),
                branch,
                result
            })
        })
        .collect::<Vec<_>>();

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
                git::delete_branch(repo_name, repo_path, branch, force_delete)
            })
            .collect::<Vec<_>>();
        results.extend(branch_deletions);
    }

    if all_child_removals_succeeded
    {
        cleanup_aggregate_worktree(&target)?;
    }

    git::print_results(results)
}

struct RemovalOutcome
{
    repo_name: String,
    repo_path: PathBuf,
    branch: Option<String>,
    result: git::GitCommandResult
}

fn child_worktree_branch(
    repo_name: &str,
    repo_path: &Path,
    child_target: &Path
) -> Result<Option<String>>
{
    let entries = git::worktree_list(repo_name, repo_path)?;
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

fn cleanup_aggregate_worktree(target: &Path) -> Result<()>
{
    let marker = target.join(".gitvmr");
    if marker.exists()
    {
        if marker.is_dir()
        {
            fs::remove_dir(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
        else
        {
            fs::remove_file(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
    }

    match fs::remove_dir(target)
    {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty =>
            Ok(()),
        Err(error) => Err(error).with_context(|| {
            format!(
                "fatal: failed to remove empty worktree directory '{}'",
                target.display()
            )
        })
    }
}

pub fn move_worktree(
    working_dir: &Path,
    path: &Path,
    new_path: &Path,
    force: u8
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let source = resolve_path(working_dir, path).clean();
    let destination = resolve_path(working_dir, new_path).clean();

    fs::create_dir_all(&destination).with_context(|| {
        format!(
            "fatal: failed to create worktree target '{}'",
            destination.display()
        )
    })?;
    fs::write(destination.join(".gitvmr"), "").with_context(|| {
        format!(
            "fatal: failed to create VMR marker in '{}'",
            destination.display()
        )
    })?;

    let results = repos
        .par_iter()
        .map(|repo| {
            git::worktree_move(
                &repo.name,
                &repo.path,
                &source.join(&repo.name),
                &destination.join(&repo.name),
                force
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)?;

    let marker = source.join(".gitvmr");
    if marker.exists()
    {
        if marker.is_dir()
        {
            fs::remove_dir(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
        else
        {
            fs::remove_file(&marker).with_context(|| {
                format!(
                    "fatal: failed to remove VMR marker '{}'",
                    marker.display()
                )
            })?;
        }
    }

    match fs::remove_dir(&source)
    {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty =>
            Ok(()),
        Err(error) => Err(error).with_context(|| {
            format!(
                "fatal: failed to remove empty worktree directory '{}'",
                source.display()
            )
        })
    }
}
