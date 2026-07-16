use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;

/// Create, list, delete or verify tags
#[derive(clap::Args)]
pub struct TagArgs
{
    /// Delete existing tags with the given names
    #[arg(short, long, requires = "tag_name")]
    pub delete: bool,

    /// The name of the tag to create, delete, or describe
    #[arg(value_name = "tagname")]
    pub tag_name: Option<String>
}

/// What a `tag` invocation asks for. Total: `delete` without a tag name is
/// rejected by the `requires` attribute above.
pub enum TagAction<'a>
{
    List,
    Create(&'a str),
    Delete(&'a str)
}

impl TagArgs
{
    pub fn action(&self) -> TagAction<'_>
    {
        match (&self.tag_name, self.delete)
        {
            (Some(tag_name), true) => TagAction::Delete(tag_name),
            (Some(tag_name), false) => TagAction::Create(tag_name),
            (None, false) => TagAction::List,
            (None, true) => unreachable!()
        }
    }
}

impl Git
{
    pub fn tag(&self, repo: &Repo, tag_name: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["tag", tag_name])?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrOnly, "git tag failed")
        )
    }

    pub fn delete_tag(&self, repo: &Repo, tag_name: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["tag", "-d", tag_name])?;

        command_result(
            repo,
            &output,
            SuccessReport::line(
                Streams::StdoutOnly,
                OnEmpty::Text("git tag deleted")
            ),
            FailureReport::line(Streams::StderrOnly, "git tag failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;
    use crate::test_support::repo;

    #[test]
    fn delete_tag_with_no_output_reports_the_deleted_fallback()
    {
        // Arrange
        let git =
            Git::with(ScriptedFake::new().on(["tag", "-d", "v1"], 0, "", ""));

        // Act
        let outcome =
            git.delete_tag(&repo("backend", "/vmr/backend"), "v1").unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "git tag deleted");
    }
}
