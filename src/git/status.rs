use crate::git::{FileChange, FileEntry, RepoStatus, git_stdout, status_head};
use crate::vmr::Repo;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn status(repo: &Repo) -> Result<Option<(Repo, RepoStatus)>>
{
    let mut staged_changes = Vec::new();
    let mut unstaged_changes = Vec::new();
    let mut untracked_files = Vec::new();

    let status_output = git_stdout(&repo.path, [
        "status",
        "--porcelain=v1",
        "-z",
        "--branch",
        "--no-ahead-behind"
    ])
    .with_context(|| {
        format!(
            "fatal: failed to read git status for '{}'",
            repo.path.display()
        )
    })?;
    let mut records = status_output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty());

    let branch_header = records
        .next()
        .context("fatal: git status did not return a branch header")?;
    let (head, initial) =
        status_head(&repo.path, "failed to read git status", branch_header)?;

    while let Some(record) = records.next()
    {
        let record = String::from_utf8_lossy(record);
        if record.len() < 4
        {
            continue;
        }

        let mut chars = record.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');
        let path = PathBuf::from(record[3..].to_owned());

        if matches!(index_status, 'R' | 'C')
            || matches!(worktree_status, 'R' | 'C')
        {
            let _old_path = records.next();
        }

        if index_status == '?' && worktree_status == '?'
        {
            untracked_files
                .push(FileEntry { path, change: FileChange::NewFile });
            continue;
        }

        if index_status == ' ' && worktree_status == 'A'
        {
            staged_changes
                .push(FileEntry { path, change: FileChange::NewFile });
            continue;
        }

        if let Some(change) = parse_status_change(index_status)
        {
            staged_changes.push(FileEntry { path: path.clone(), change });
        }
        if let Some(change) = parse_status_change(worktree_status)
        {
            unstaged_changes.push(FileEntry { path, change });
        }
    }

    Ok(Some((repo.clone(), RepoStatus {
        head,
        initial,
        staged_changes,
        unstaged_changes,
        untracked_files
    })))
}

fn parse_status_change(status: char) -> Option<FileChange>
{
    match status
    {
        'A' => Some(FileChange::NewFile),
        'M' => Some(FileChange::Modified),
        'D' => Some(FileChange::Deleted),
        'R' => Some(FileChange::Renamed),
        'T' => Some(FileChange::TypeChange),
        'C' => Some(FileChange::Copied),
        'U' => Some(FileChange::Modified),
        _ => None
    }
}
