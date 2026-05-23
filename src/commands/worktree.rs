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
    let branch = match (branch, commit_ish)
    {
        (Some(branch), _) => Some(branch.to_owned()),
        (None, Some(_)) => None,
        (None, None) => Some(
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
            git::worktree_add(
                &repo.name,
                &repo.path,
                &target.join(&repo.name),
                branch.as_deref(),
                commit_ish
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)
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
        (&a.state, short_head(&a.head), &a.repo).cmp(&(
            &b.state,
            short_head(&b.head),
            &b.repo
        ))
    });

    let mut state_groups: BTreeMap<
        (git::ChildWorktreeState, String),
        Vec<String>
    > = BTreeMap::new();

    for entry in entries
    {
        state_groups
            .entry((entry.state, short_head(&entry.head).to_owned()))
            .or_default()
            .push(entry.repo);
    }

    for ((state, head), mut repos) in state_groups
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

        println!(
            "{} {} {}{}",
            root.display(),
            head,
            render_state(&state),
            suffix
        );
    }
}

fn render_state(state: &git::ChildWorktreeState) -> String
{
    match state
    {
        git::ChildWorktreeState::Branch(branch) => format!("[{branch}]"),
        git::ChildWorktreeState::Detached => "(detached HEAD)".to_owned()
    }
}

fn short_head(head: &str) -> &str
{
    head.get(..8).unwrap_or(head)
}

pub fn remove(working_dir: &Path, path: &Path, force: u8) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let target = resolve_path(working_dir, path).clean();

    let results = repos
        .par_iter()
        .map(|repo| {
            git::worktree_remove(
                &repo.name,
                &repo.path,
                &target.join(&repo.name),
                force
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)?;

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

    match fs::remove_dir(&target)
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
