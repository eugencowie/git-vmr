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
}
