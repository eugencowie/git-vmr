use crate::git::{Git, GitOutput, stderr};
use anyhow::{Context, Result, bail};
use std::path::Path;

/// Every detached head shortens its hash by this one rule, whichever source
/// it was resolved from.
const SHORT_HASH_LEN: usize = 8;

/// Where a child repo currently points: a branch, an unborn branch (no
/// commits yet), or a detached commit's short hash. Only the status header
/// can observe unbornness; sources that cannot report a plain branch.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Head
{
    Branch(String),
    Unborn(String),
    Detached(String)
}

impl Head
{
    /// Builds a head from `worktree list --porcelain` fields: the branch
    /// field when present, otherwise the record's full HEAD hash shortened.
    pub(crate) fn from_worktree_record(
        branch: Option<String>,
        full_hash: &str
    ) -> Self
    {
        match branch
        {
            Some(branch) => Self::Branch(branch),
            None => Self::Detached(
                full_hash.get(..SHORT_HASH_LEN).unwrap_or(full_hash).to_owned()
            )
        }
    }
}

impl Git
{
    /// Resolves a repo's head directly: the branch `symbolic-ref` names, or
    /// the detached commit when it answers with exit code 1.
    pub(crate) fn head(&self, repo_path: &Path) -> Result<Head>
    {
        match self.output(repo_path, [
            "symbolic-ref",
            "--quiet",
            "--short",
            "HEAD"
        ])?
        {
            GitOutput { status, stdout, stderr: _ } if status.success() =>
                Ok(Head::Branch(
                    String::from_utf8_lossy(&stdout).trim().to_owned()
                )),
            GitOutput { status, .. } if status.code() == Some(1) =>
                self.detached_head(repo_path),
            output => bail!(
                "fatal: failed to resolve HEAD for '{}': {}",
                repo_path.display(),
                stderr(&output)
            )
        }
    }

    /// Resolves a repo's head from the branch header git status already
    /// returned; only detached heads cost a second invocation.
    pub(crate) fn head_from_status_header(
        &self,
        repo_path: &Path,
        context: &str,
        header: &[u8]
    ) -> Result<Head>
    {
        let header = String::from_utf8_lossy(header);
        let header = header
            .strip_prefix("## ")
            .context("fatal: git status branch header had unexpected format")?;

        if let Some(branch) = header.strip_prefix("No commits yet on ")
        {
            return Ok(Head::Unborn(branch.to_owned()));
        }

        if header == "HEAD (no branch)" || header.starts_with("HEAD detached")
        {
            return self.detached_head(repo_path).with_context(|| {
                format!("fatal: {context} for '{}'", repo_path.display())
            });
        }

        let branch = header.split("...").next().unwrap_or(header).to_owned();
        Ok(Head::Branch(branch))
    }

    fn detached_head(&self, repo_path: &Path) -> Result<Head>
    {
        let hash = String::from_utf8_lossy(&self.stdout(repo_path, [
            "rev-parse",
            &format!("--short={SHORT_HASH_LEN}"),
            "HEAD"
        ])?)
        .trim()
        .to_owned();

        Ok(Head::Detached(hash))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;

    const REPO: &str = "/vmr/backend";

    #[test]
    fn head_names_the_branch_symbolic_ref_answers()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["symbolic-ref", "--quiet", "--short", "HEAD"],
            0,
            "develop\n",
            ""
        ));

        // Act
        let head = git.head(Path::new(REPO)).unwrap();

        // Assert
        assert_eq!(head, Head::Branch("develop".to_owned()));
    }

    #[test]
    fn head_resolves_detached_through_the_shared_fallback()
    {
        // Arrange
        let git = Git::with(
            ScriptedFake::new()
                .on(["symbolic-ref", "--quiet", "--short", "HEAD"], 1, "", "")
                .on(["rev-parse", "--short=8", "HEAD"], 0, "abc12345\n", "")
        );

        // Act
        let head = git.head(Path::new(REPO)).unwrap();

        // Assert
        assert_eq!(head, Head::Detached("abc12345".to_owned()));
    }

    #[test]
    fn head_reports_symbolic_ref_failures()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["symbolic-ref", "--quiet", "--short", "HEAD"],
            128,
            "",
            "fatal: not a git repository"
        ));

        // Act
        let error = git.head(Path::new(REPO)).err().unwrap();

        // Assert
        assert!(error.to_string().contains("failed to resolve HEAD"));
        assert!(error.to_string().contains("not a git repository"));
    }

    #[test]
    fn status_header_names_the_branch_before_any_divergence_marker()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new());

        // Act
        let head = git
            .head_from_status_header(
                Path::new(REPO),
                "failed to read git status",
                b"## main...origin/main"
            )
            .unwrap();

        // Assert
        assert_eq!(head, Head::Branch("main".to_owned()));
    }

    #[test]
    fn status_header_reports_an_unborn_branch()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new());

        // Act
        let head = git
            .head_from_status_header(
                Path::new(REPO),
                "failed to read git status",
                b"## No commits yet on main"
            )
            .unwrap();

        // Assert
        assert_eq!(head, Head::Unborn("main".to_owned()));
    }

    #[test]
    fn status_header_resolves_detached_through_the_shared_fallback()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["rev-parse", "--short=8", "HEAD"],
            0,
            "abc12345\n",
            ""
        ));

        // Act
        let head = git
            .head_from_status_header(
                Path::new(REPO),
                "failed to read git status",
                b"## HEAD (no branch)"
            )
            .unwrap();

        // Assert
        assert_eq!(head, Head::Detached("abc12345".to_owned()));
    }

    #[test]
    fn status_header_rejects_unexpected_formats()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new());

        // Act
        let error = git
            .head_from_status_header(
                Path::new(REPO),
                "failed to read git status",
                b"?? not-a-header"
            )
            .err()
            .unwrap();

        // Assert
        assert!(error.to_string().contains("unexpected format"));
    }

    #[test]
    fn worktree_record_with_a_branch_field_is_on_that_branch()
    {
        assert_eq!(
            Head::from_worktree_record(
                Some("feature".to_owned()),
                "1234567890abcdef"
            ),
            Head::Branch("feature".to_owned())
        );
    }

    #[test]
    fn worktree_record_without_a_branch_field_shortens_the_hash()
    {
        assert_eq!(
            Head::from_worktree_record(None, "1234567890abcdef"),
            Head::Detached("12345678".to_owned())
        );
    }
}
