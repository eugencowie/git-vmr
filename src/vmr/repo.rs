use crate::git::{GitOutput, git_output, git_stdout};
use anyhow::{Context, Result, bail};
use std::path::PathBuf;

#[derive(Clone, PartialEq, Eq)]
pub enum Head
{
    Branch(String),
    Detached(String)
}

#[derive(Clone, PartialEq, Eq)]
pub struct RepoBranches
{
    pub branches: Vec<String>,
    pub head: Head
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileChange
{
    NewFile,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Copied
}

#[derive(Clone, PartialEq, Eq)]
pub struct FileEntry
{
    pub path: PathBuf,
    pub change: FileChange
}

#[derive(Clone, PartialEq, Eq)]
pub struct RepoStatus
{
    pub head: Head,
    pub initial: bool,
    pub staged_changes: Vec<FileEntry>,
    pub unstaged_changes: Vec<FileEntry>,
    pub untracked_files: Vec<FileEntry>
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Repo
{
    pub name: String,
    pub path: PathBuf
}

impl Repo
{
    pub fn new(name: String, path: PathBuf) -> Self
    {
        Self { name, path }
    }

    pub fn find(path: PathBuf) -> Option<Repo>
    {
        // Check for repository marker
        if !path.join(".git").exists()
        {
            return None;
        }

        // Use directory name as repository name
        let name = path.file_name().and_then(|name| name.to_str())?.to_owned();

        Some(Repo::new(name, path))
    }

    pub fn is_dirty(&self) -> Result<bool>
    {
        let output = git_output(&self.path, ["diff", "--cached", "--quiet"])
            .with_context(|| {
                format!(
                    "failed to inspect staged changes for '{}'",
                    self.path.display()
                )
            })?;

        match output.status.code()
        {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => bail!(
                "git diff --cached --quiet failed for '{}': {}",
                self.path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            )
        }
    }

    pub fn branches(&self) -> Result<Option<(String, RepoBranches)>>
    {
        let branches_output = git_stdout(&self.path, [
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads"
        ])
        .with_context(|| {
            format!(
                "failed to read branch information for '{}'",
                self.path.display()
            )
        })?;
        let branches = String::from_utf8_lossy(&branches_output)
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let head = match git_output(&self.path, [
            "symbolic-ref",
            "--quiet",
            "--short",
            "HEAD"
        ])
        .with_context(|| {
            format!(
                "failed to read branch information for '{}'",
                self.path.display()
            )
        })?
        {
            GitOutput { status, stdout, stderr: _ } if status.success() =>
                Head::Branch(String::from_utf8_lossy(&stdout).trim().to_owned()),
            GitOutput { status, .. } if status.code() == Some(1) =>
            {
                let hash = String::from_utf8_lossy(
                    &git_stdout(&self.path, ["rev-parse", "--short", "HEAD"])
                        .with_context(|| {
                        format!(
                            "failed to read branch information for '{}'",
                            self.path.display()
                        )
                    })?
                )
                .trim()
                .to_owned();
                Head::Detached(hash)
            }
            GitOutput { stderr, .. } => bail!(
                "failed to read branch information for '{}': {}",
                self.path.display(),
                String::from_utf8_lossy(&stderr).trim()
            )
        };

        Ok(Some((self.name.clone(), RepoBranches { branches, head })))
    }

    pub fn status(&self) -> Result<Option<(Repo, RepoStatus)>>
    {
        let mut staged_changes = Vec::new();
        let mut unstaged_changes = Vec::new();
        let mut untracked_files = Vec::new();

        // Read porcelain status
        let status_output = git_stdout(&self.path, [
            "status",
            "--porcelain=v1",
            "-z",
            "--branch",
            "--no-ahead-behind"
        ])
        .with_context(|| {
            format!("failed to read git status for '{}'", self.path.display())
        })?;
        let mut records = status_output
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty());

        // Parse branch header
        let branch_header = records
            .next()
            .context("git status did not return a branch header")?;
        let (head, initial) = parse_branch_header(self, branch_header)?;

        // Parse path records
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

            // Skip the old path record for renamed or copied files
            if matches!(index_status, 'R' | 'C')
                || matches!(worktree_status, 'R' | 'C')
            {
                let _old_path = records.next();
            }

            // Track untracked files separately
            if index_status == '?' && worktree_status == '?'
            {
                untracked_files
                    .push(FileEntry { path, change: FileChange::NewFile });
                continue;
            }

            // Treat intent-to-add as staged for VMR status output
            if index_status == ' ' && worktree_status == 'A'
            {
                staged_changes
                    .push(FileEntry { path, change: FileChange::NewFile });
                continue;
            }

            // Add index and worktree changes
            if let Some(change) = parse_status_change(index_status)
            {
                staged_changes.push(FileEntry { path: path.clone(), change });
            }
            if let Some(change) = parse_status_change(worktree_status)
            {
                unstaged_changes.push(FileEntry { path, change });
            }
        }

        Ok(Some((self.clone(), RepoStatus {
            head,
            initial,
            staged_changes,
            unstaged_changes,
            untracked_files
        })))
    }
}

fn parse_branch_header(repo: &Repo, header: &[u8]) -> Result<(Head, bool)>
{
    // Decode and validate branch header
    let header = String::from_utf8_lossy(header);
    let header = header
        .strip_prefix("## ")
        .context("git status branch header had unexpected format")?;

    // Detect unborn branch
    if let Some(branch) = header.strip_prefix("No commits yet on ")
    {
        return Ok((Head::Branch(branch.to_owned()), true));
    }

    // Detect detached HEAD
    if header == "HEAD (no branch)" || header.starts_with("HEAD detached")
    {
        let hash = String::from_utf8_lossy(
            &git_stdout(&repo.path, ["rev-parse", "--short", "HEAD"])
                .with_context(|| {
                    format!(
                        "failed to read git status for '{}'",
                        repo.path.display()
                    )
                })?
        )
        .trim()
        .to_owned();
        return Ok((Head::Detached(hash), false));
    }

    // Parse branch name
    let branch = header.split("...").next().unwrap_or(header).to_owned();
    Ok((Head::Branch(branch), false))
}

fn parse_status_change(status: char) -> Option<FileChange>
{
    // Map porcelain status code
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
