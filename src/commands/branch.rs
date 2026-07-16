use crate::cli::CliContext;
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: BranchArgs
) -> Result<Rendered>
{
    match args.action()
    {
        BranchAction::List => branches(workspace),

        // Branch in each child repository
        BranchAction::Create(branch_name) =>
            workspace.run(|git, repo| git.branch(repo, branch_name)),

        // Delete branch in each child repository
        BranchAction::Delete { branch_name, force } => workspace
            .run(|git, repo| git.delete_branch(repo, branch_name, force))
    }
}

fn branches(workspace: &Workspace) -> Result<Rendered>
{
    // Collect branch information from child repositories, in repo order
    let branches = workspace
        .map(|git, repo| git.branches(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render results
    Ok(render::branches(&branches).into())
}

use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult, RepoBranches};
use crate::vmr::Repo;
use anyhow::Context;

/// List, create, or delete branches
#[derive(clap::Args)]
pub struct BranchArgs
{
    /// Delete a branch. The branch must be fully merged in its upstream
    /// branch
    #[arg(
        short,
        long,
        conflicts_with = "force_delete",
        requires = "branch_name"
    )]
    pub delete: bool,

    /// Shortcut for `--delete --force`
    #[arg(short = 'D', conflicts_with = "delete", requires = "branch_name")]
    pub force_delete: bool,

    /// In combination with `-d` (or `--delete`), allow deleting the branch
    /// irrespective of its merged status, or whether it even points to a
    /// valid commit
    #[arg(
        short,
        long,
        conflicts_with = "force_delete",
        requires_all = ["branch_name", "delete"]
    )]
    pub force: bool,

    /// Creates a new branch head named [branch-name] which points to the
    /// current HEAD
    #[arg(value_name = "branch-name")]
    pub branch_name: Option<String>
}

/// What a `branch` invocation asks for. Total: the flag combinations the
/// arg attributes above reject cannot reach this enum.
pub enum BranchAction<'a>
{
    List,
    Create(&'a str),
    Delete
    {
        branch_name: &'a str,
        force: bool
    }
}

impl BranchArgs
{
    fn action(&self) -> BranchAction<'_>
    {
        match &self.branch_name
        {
            Some(branch_name) if self.delete || self.force_delete =>
                BranchAction::Delete {
                    branch_name,
                    force: self.force || self.force_delete
                },
            Some(branch_name) => BranchAction::Create(branch_name),
            None => BranchAction::List
        }
    }
}

impl Git
{
    fn branch(&self, repo: &Repo, branch_name: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["branch", branch_name])?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrOnly, "git branch failed")
        )
    }

    fn branches(&self, repo: &Repo) -> Result<Option<(String, RepoBranches)>>
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
}
