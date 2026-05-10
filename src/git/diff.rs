use crate::git::{git_output, stderr};
use anyhow::{Context, Result, bail};
use std::path::Path;

pub fn is_dirty(repo_path: &Path) -> Result<bool>
{
    let output = git_output(repo_path, ["diff", "--cached", "--quiet"])
        .with_context(|| {
            format!(
                "failed to inspect staged changes for '{}'",
                repo_path.display()
            )
        })?;

    match output.status.code()
    {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => bail!(
            "git diff --cached --quiet failed for '{}': {}",
            repo_path.display(),
            stderr(&output)
        )
    }
}
