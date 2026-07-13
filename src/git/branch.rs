use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult, RepoBranches};
use crate::vmr::Repo;
use anyhow::{Context, Result, bail};
use std::path::Path;

impl Git
{
    pub fn branch(
        &self,
        repo_name: &str,
        repo_path: &Path,
        branch_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["branch", branch_name])?;

        command_result(
            repo_name,
            repo_path,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrOnly, "git branch failed")
        )
    }

    pub fn delete_branch(
        &self,
        repo_name: &str,
        repo_path: &Path,
        branch_name: &str,
        force: bool
    ) -> GitCommandResult
    {
        let flag = if force { "-D" } else { "-d" };
        let output = self.output(repo_path, ["branch", flag, branch_name])?;

        command_result(
            repo_name,
            repo_path,
            &output,
            SuccessReport::fixed(format!("Deleted branch {branch_name}")),
            FailureReport::line(Streams::StderrOnly, "git branch failed")
        )
    }

    pub fn branch_exists(
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

    pub fn branches(
        &self,
        repo: &Repo
    ) -> Result<Option<(String, RepoBranches)>>
    {
        let branches_output = self
            .stdout(&repo.path, [
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/heads"
            ])
            .with_context(|| {
                format!(
                    "fatal: failed to read branch information for '{}'",
                    repo.path.display()
                )
            })?;
        let branches = String::from_utf8_lossy(&branches_output)
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let head = self.head(&repo.path).with_context(|| {
            format!(
                "fatal: failed to read branch information for '{}'",
                repo.path.display()
            )
        })?;

        Ok(Some((repo.name.clone(), RepoBranches { branches, head })))
    }

    pub fn tags(&self, repo: &Repo) -> Result<Option<(String, Vec<String>)>>
    {
        let tags_output = self
            .stdout(&repo.path, [
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/tags"
            ])
            .with_context(|| {
                format!(
                    "fatal: failed to read tag information for '{}'",
                    repo.path.display()
                )
            })?;
        let tags = String::from_utf8_lossy(&tags_output)
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        Ok(Some((repo.name.clone(), tags)))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Head, ScriptedFake};
    use std::path::PathBuf;

    fn repo() -> Repo
    {
        Repo { name: "backend".to_owned(), path: PathBuf::from("/vmr/backend") }
    }

    #[test]
    fn branches_resolves_detached_head_through_rev_parse_fallback()
    {
        // Arrange
        let git = Git::with(
            ScriptedFake::new()
                .on(
                    ["for-each-ref", "--format=%(refname:short)", "refs/heads"],
                    0,
                    "develop\nmain\n",
                    ""
                )
                .on(["symbolic-ref", "--quiet", "--short", "HEAD"], 1, "", "")
                .on(["rev-parse", "--short=8", "HEAD"], 0, "abc12345\n", "")
        );

        // Act
        let (name, branches) = git.branches(&repo()).unwrap().unwrap();

        // Assert
        assert_eq!(name, "backend");
        assert_eq!(branches.branches, vec!["develop", "main"]);
        assert!(
            matches!(&branches.head, Head::Detached(hash) if hash == "abc12345")
        );
    }

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
