use crate::git::{Git, GitCommandResult, command_result, first_non_empty_line};
use std::path::Path;

impl Git
{
    pub fn tag(
        &self,
        repo_name: &str,
        repo_path: &Path,
        tag_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["tag", tag_name])?;

        command_result(
            repo_name,
            &output,
            |_| None,
            |output| first_non_empty_line(&output.stderr, "git tag failed")
        )
    }

    pub fn delete_tag(
        &self,
        repo_name: &str,
        repo_path: &Path,
        tag_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["tag", "-d", tag_name])?;

        command_result(
            repo_name,
            &output,
            |output| {
                first_non_empty_line(&output.stdout, "git tag delete succeeded")
                    .into()
            },
            |output| first_non_empty_line(&output.stderr, "git tag failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;

    #[test]
    fn delete_tag_with_no_output_reports_success()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["tag", "-d", "v1.0.0"],
            0,
            "",
            ""
        ));

        // Act
        let outcome = git
            .delete_tag("backend", Path::new("/vmr/backend"), "v1.0.0")
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "git tag delete succeeded");
    }
}
