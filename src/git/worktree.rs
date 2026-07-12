use crate::git::{
    Git, GitCommandResult, command_result, first_non_empty_line_with_fallback
};
use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildWorktree
{
    pub path: PathBuf,
    pub head: String,
    pub state: ChildWorktreeState,
    pub bare: bool,
    pub locked: bool,
    pub prunable: bool
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ChildWorktreeState
{
    Branch(String),
    Detached
}

impl Git
{
    pub fn worktree_add(
        &self,
        repo_name: &str,
        repo_path: &Path,
        target: &Path,
        branch: Option<&str>,
        commit_ish: Option<&str>
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("worktree"), OsString::from("add")];

        if let Some(branch) = branch
        {
            args.push(OsString::from("-b"));
            args.push(OsString::from(branch));
        }

        args.push(target.as_os_str().to_owned());

        if let Some(commit_ish) = commit_ish
        {
            args.push(OsString::from(commit_ish));
        }

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                Some(first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git worktree add succeeded"
                ))
            },
            failure_line
        )
    }

    pub fn worktree_remove(
        &self,
        repo_name: &str,
        repo_path: &Path,
        target: &Path,
        force: u8
    ) -> GitCommandResult
    {
        let mut args =
            vec![OsString::from("worktree"), OsString::from("remove")];

        for _ in 0..force
        {
            args.push(OsString::from("-f"));
        }

        args.push(target.as_os_str().to_owned());

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stdout,
                    &output.stderr,
                    "git worktree remove succeeded"
                );

                if message == "git worktree remove succeeded"
                {
                    None
                }
                else
                {
                    Some(message)
                }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git worktree remove failed"
                )
            }
        )
    }

    pub fn worktree_move(
        &self,
        repo_name: &str,
        repo_path: &Path,
        source: &Path,
        destination: &Path,
        force: u8
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("worktree"), OsString::from("move")];

        for _ in 0..force
        {
            args.push(OsString::from("-f"));
        }

        args.push(source.as_os_str().to_owned());
        args.push(destination.as_os_str().to_owned());

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stdout,
                    &output.stderr,
                    "git worktree move succeeded"
                );

                if message == "git worktree move succeeded"
                {
                    None
                }
                else
                {
                    Some(message)
                }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git worktree move failed"
                )
            }
        )
    }

    pub fn worktree_list(
        &self,
        repo_name: &str,
        repo_path: &Path
    ) -> Result<Vec<ChildWorktree>>
    {
        let output = self.output(repo_path, [
            OsString::from("worktree"),
            OsString::from("list"),
            OsString::from("--porcelain"),
            OsString::from("-z")
        ])?;

        if !output.status.success()
        {
            bail!(
                "fatal: failed to list worktrees for '{}': {}",
                repo_name,
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git worktree list failed"
                )
            );
        }

        parse_worktree_list(repo_name, &output.stdout)
    }
}

fn parse_worktree_list(
    repo_name: &str,
    output: &[u8]
) -> Result<Vec<ChildWorktree>>
{
    let mut entries = Vec::new();
    let mut current = WorktreeRecord::default();

    for field in output.split(|byte| *byte == 0)
    {
        if field.is_empty()
        {
            if !current.is_empty()
            {
                entries.push(current.finish(repo_name)?);
                current = WorktreeRecord::default();
            }
            continue;
        }

        if let Some(value) = field.strip_prefix(b"worktree ")
        {
            if !current.is_empty()
            {
                entries.push(current.finish(repo_name)?);
                current = WorktreeRecord::default();
            }
            current.path = Some(PathBuf::from(
                String::from_utf8_lossy(value).into_owned()
            ));
        }
        else if let Some(value) = field.strip_prefix(b"HEAD ")
        {
            current.head = Some(String::from_utf8_lossy(value).into_owned());
        }
        else if let Some(value) = field.strip_prefix(b"branch ")
        {
            let branch = String::from_utf8_lossy(value);
            current.branch = Some(
                branch
                    .strip_prefix("refs/heads/")
                    .unwrap_or(&branch)
                    .to_owned()
            );
        }
        else if field == b"detached"
        {
            current.detached = true;
        }
        else if field == b"bare"
        {
            current.bare = true;
        }
        else if field.starts_with(b"locked")
        {
            current.locked = true;
        }
        else if field.starts_with(b"prunable")
        {
            current.prunable = true;
        }
    }

    if !current.is_empty()
    {
        entries.push(current.finish(repo_name)?);
    }

    Ok(entries)
}

#[derive(Default)]
struct WorktreeRecord
{
    path: Option<PathBuf>,
    head: Option<String>,
    branch: Option<String>,
    detached: bool,
    bare: bool,
    locked: bool,
    prunable: bool
}

impl WorktreeRecord
{
    fn is_empty(&self) -> bool
    {
        self.path.is_none()
            && self.head.is_none()
            && self.branch.is_none()
            && !self.detached
            && !self.bare
            && !self.locked
            && !self.prunable
    }

    fn finish(self, repo_name: &str) -> Result<ChildWorktree>
    {
        let path = self.path.with_context(|| {
            format!("fatal: malformed worktree list for '{repo_name}': missing worktree path")
        })?;
        let head = self.head.with_context(|| {
            format!(
                "fatal: malformed worktree list for '{repo_name}': missing HEAD"
            )
        })?;
        let state = match (self.branch, self.detached)
        {
            (Some(branch), _) => ChildWorktreeState::Branch(branch),
            (None, true) => ChildWorktreeState::Detached,
            (None, false) => ChildWorktreeState::Detached
        };

        Ok(ChildWorktree {
            path,
            head,
            state,
            bare: self.bare,
            locked: self.locked,
            prunable: self.prunable
        })
    }
}

fn failure_line(output: &crate::git::GitOutput) -> String
{
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            first_non_empty_line_with_fallback(
                &output.stdout,
                &output.stderr,
                "git worktree add failed"
            )
        })
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn parses_branch_detached_flags_and_unknown_fields()
    {
        let entries = parse_worktree_list(
            "backend",
            b"worktree /repo/backend\0HEAD 1234567890abcdef\0branch refs/heads/main\0unknown value\0\0worktree /wt/backend\0HEAD abcdef1234567890\0detached\0locked reason\0prunable stale\0\0"
        )
        .unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, PathBuf::from("/repo/backend"));
        assert_eq!(
            entries[0].state,
            ChildWorktreeState::Branch("main".to_owned())
        );
        assert!(!entries[0].locked);
        assert_eq!(entries[1].state, ChildWorktreeState::Detached);
        assert!(entries[1].locked);
        assert!(entries[1].prunable);
    }

    #[test]
    fn reports_malformed_records_with_repository_context()
    {
        let error =
            parse_worktree_list("frontend", b"worktree /repo/frontend\0\0")
                .err()
                .unwrap();

        assert!(
            error
                .to_string()
                .contains("malformed worktree list for 'frontend'")
        );
    }
}
