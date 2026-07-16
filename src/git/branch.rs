use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use anyhow::{Result, bail};
use std::path::Path;

// Shared branch primitives: both the branch command and the worktree-root
// lifecycle delete branches and probe for their existence, which is what
// earns these methods a place in the git core.
impl Git
{
    pub(crate) fn delete_branch(
        &self,
        repo: &Repo,
        branch_name: &str,
        force: bool
    ) -> GitCommandResult
    {
        let flag = if force { "-D" } else { "-d" };
        let output = self.output(&repo.path, ["branch", flag, branch_name])?;

        command_result(
            repo,
            &output,
            SuccessReport::fixed(format!("Deleted branch {branch_name}")),
            FailureReport::line(Streams::StderrOnly, "git branch failed")
        )
    }

    pub(crate) fn branch_exists(
        &self,
        repo_path: &Path,
        branch_name: &str
    ) -> Result<bool>
    {
        let ref_name = format!("refs/heads/{branch_name}");
        let output =
            self.output(repo_path, ["show-ref", "--exists", &ref_name])?;

        match output.status.code()
        {
            Some(0) => Ok(true),
            Some(2) => Ok(false),
            _ => bail!(
                "fatal: failed to check branch '{}' in '{}': {}",
                branch_name,
                repo_path.display(),
                Streams::StderrOnly.first_line(&output, "git show-ref failed")
            )
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;

    #[test]
    fn branch_exists_maps_show_ref_exit_codes()
    {
        for (code, expected) in [(0, true), (2, false)]
        {
            // Arrange
            let git = Git::with(ScriptedFake::new().on(
                ["show-ref", "--exists", "refs/heads/main"],
                code,
                "",
                ""
            ));

            // Act
            let exists =
                git.branch_exists(Path::new("/vmr/backend"), "main").unwrap();

            // Assert
            assert_eq!(exists, expected);
        }
    }
}
