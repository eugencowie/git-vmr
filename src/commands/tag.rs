use crate::cli::CliContext;
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::{Context, Result};

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: TagArgs
) -> Result<Rendered>
{
    match args.action()
    {
        TagAction::List => tags(workspace),

        // Create tag in each child repository
        TagAction::Create(tag_name) =>
            workspace.run(|git, repo| git.tag(repo, tag_name)),

        // Delete tag in each child repository
        TagAction::Delete(tag_name) =>
            workspace.run(|git, repo| git.delete_tag(repo, tag_name)),
    }
}

fn tags(workspace: &Workspace) -> Result<Rendered>
{
    // Collect tag information from child repositories, in repo order
    let tags = workspace
        .map(|git, repo| git.tags(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render results
    Ok(render::tags(&tags).into())
}

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
    fn action(&self) -> TagAction<'_>
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
    fn tags(&self, repo: &Repo) -> Result<Option<(String, Vec<String>)>>
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

    fn tag(&self, repo: &Repo, tag_name: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["tag", tag_name])?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrOnly, "git tag failed")
        )
    }

    fn delete_tag(&self, repo: &Repo, tag_name: &str) -> GitCommandResult
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
    use crate::git::{RepoOutcome, ScriptedFake};
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
