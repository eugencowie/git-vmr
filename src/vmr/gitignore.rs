use anyhow::{Context, Result};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

/// The root repo's `.gitignore`, as git-vmr reads and writes it: each child
/// repo is ignored by an anchored `/<path>/` line.
///
/// Not `.git/info/exclude`: that file is shared by every worktree of the
/// root repo while manifests are per branch, so one worktree's ignore lines
/// would be another's too.
pub struct Gitignore
{
    path: PathBuf,
    text: String
}

impl Gitignore
{
    /// The `.gitignore` at `root`, empty if there is none.
    pub fn read(root: &Path) -> Gitignore
    {
        let path = root.join(".gitignore");
        let text = fs::read_to_string(&path).unwrap_or_default();
        Gitignore { path, text }
    }

    /// The line that ignores the child repo at `path`.
    pub fn line(path: &str) -> String
    {
        format!("/{path}/")
    }

    /// Whether the file has the exact line that ignores the child repo at
    /// `path`.
    pub fn has_line(&self, path: &str) -> bool
    {
        let line = Gitignore::line(path);
        self.text.lines().any(|candidate| candidate.trim_end() == line)
    }

    /// Appends a line for each of `paths` that lacks one, keeping the rest
    /// of the file as it is, and writes the file if anything was added.
    pub fn ignore<'p>(
        mut self,
        paths: impl IntoIterator<Item = &'p str>
    ) -> Result<()>
    {
        // Find the paths that lack a line
        let missing = paths
            .into_iter()
            .filter(|path| !self.has_line(path))
            .collect::<Vec<_>>();
        if missing.is_empty()
        {
            return Ok(());
        }

        // Append the lines, ending an unterminated last line first
        if !self.text.is_empty() && !self.text.ends_with('\n')
        {
            self.text.push('\n');
        }
        for path in missing
        {
            let _ = writeln!(self.text, "{}", Gitignore::line(path));
        }

        // Write file
        fs::write(&self.path, &self.text).with_context(|| {
            format!("fatal: failed to write '{}'", self.path.display())
        })
    }
}
