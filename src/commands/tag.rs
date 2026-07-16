use crate::cli::CliContext;
use crate::render::{Rendered, SuffixPolicy, repo_list_suffix};
use crate::workspace::Workspace;
use anyhow::{Context, Result};

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &TagArgs
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
    Ok(render(&tags).into())
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

use std::collections::{BTreeMap, BTreeSet};

/// Renders the tag list grouped across child repos, in tag name order.
fn render(repos: &[(String, Vec<String>)]) -> String
{
    let mut output = String::new();
    let repo_count = repos.len();
    let mut tag_groups: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();

    for (repo, tags) in repos
    {
        for tag in tags
        {
            tag_groups.entry(tag.as_str()).or_default().insert(repo.as_str());
        }
    }

    for (tag, tag_repos) in tag_groups
    {
        output.push_str(tag);

        let repo_names = tag_repos.into_iter().collect::<Vec<_>>();
        output.push_str(&repo_list_suffix(
            &repo_names,
            repo_count,
            SuffixPolicy::Truncated
        ));

        output.push('\n');
    }

    output
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

#[cfg(test)]
mod render_tests
{
    use super::*;

    #[test]
    fn renders_shared_tag_without_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), vec!["v1.0.0".to_owned()]),
            ("frontend".to_owned(), vec!["v1.0.0".to_owned()]),
        ];

        let output = render(&repos);

        assert_eq!(output, "v1.0.0\n");
    }

    #[test]
    fn renders_partial_tag_with_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), vec!["v1.0.0".to_owned()]),
            ("frontend".to_owned(), vec![
                "v1.0.0".to_owned(),
                "v1.1.0".to_owned(),
            ]),
        ];

        let output = render(&repos);

        assert!(output.contains("v1.0.0\n"));
        assert!(output.contains("v1.1.0 \x1b[90m(frontend)\x1b[0m\n"));
    }

    #[test]
    fn renders_partial_tag_with_sorted_repo_list()
    {
        let repos = vec![
            ("backend".to_owned(), vec!["v1.2.0".to_owned()]),
            ("frontend".to_owned(), vec!["v1.2.0".to_owned()]),
            ("tools".to_owned(), vec![]),
        ];

        let output = render(&repos);

        assert_eq!(output, "v1.2.0 \x1b[90m(backend, frontend)\x1b[0m\n");
    }

    #[test]
    fn renders_tags_in_name_order()
    {
        let repos = vec![
            ("backend".to_owned(), vec![
                "v2.0.0".to_owned(),
                "v1.0.0".to_owned(),
            ]),
            ("frontend".to_owned(), vec!["v1.1.0".to_owned()]),
        ];

        let output = render(&repos);

        assert_eq!(
            output,
            concat!(
                "v1.0.0 \x1b[90m(backend)\x1b[0m\n",
                "v1.1.0 \x1b[90m(frontend)\x1b[0m\n",
                "v2.0.0 \x1b[90m(backend)\x1b[0m\n"
            )
        );
    }

    #[test]
    fn renders_empty_output_for_no_repos_or_tag_refs()
    {
        assert_eq!(render(&[]), "");

        let repos = vec![("backend".to_owned(), vec![])];
        assert_eq!(render(&repos), "");
    }
}
