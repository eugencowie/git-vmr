use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
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
            repo_path,
            &output,
            SuccessReport::Quiet,
            FailureReport::Line {
                from: Streams::StderrOnly,
                fallback: "git tag failed"
            }
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
            repo_path,
            &output,
            SuccessReport::Line {
                from: Streams::StdoutOnly,
                on_empty: OnEmpty::Text("git tag deleted")
            },
            FailureReport::Line {
                from: Streams::StderrOnly,
                fallback: "git tag failed"
            }
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
    fn delete_tag_with_no_output_reports_the_deleted_fallback()
    {
        // Arrange
        let git =
            Git::with(ScriptedFake::new().on(["tag", "-d", "v1"], 0, "", ""));

        // Act
        let outcome =
            git.delete_tag("backend", Path::new("/vmr/backend"), "v1").unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "git tag deleted");
    }
}
