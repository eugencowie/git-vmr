use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &FetchArgs
) -> Result<Rendered>
{
    // Fetch in each child repository
    workspace.run(|git, repo| git.fetch(repo, args))
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;

/// Download objects and refs from another repository
#[derive(clap::Args)]
pub struct FetchArgs
{
    /// The "remote" repository that is the source of a fetch or pull
    /// operation
    #[arg(value_name = "repository")]
    pub repository: Option<String>,

    /// Specifies which refs to fetch and which local refs to update
    #[arg(value_name = "refspec")]
    pub refspecs: Vec<String>
}

impl Git
{
    fn fetch(&self, repo: &Repo, fetch: &FetchArgs) -> GitCommandResult
    {
        let mut args = vec![OsString::from("fetch")];

        if let Some(repository) = &fetch.repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(fetch.refspecs.iter().map(OsString::from));

        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::line(Streams::StderrThenStdout, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "git fetch failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, ScriptedFake};
    use crate::test_support::{cli_context, vmr_fixture};
    use std::sync::Arc;

    #[test]
    fn fetches_in_every_child_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(["fetch"], 0, "", ""));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = run(&workspace, &cli_context(tmp.path()), &FetchArgs {
            repository: None,
            refspecs: vec![]
        });

        // Assert
        assert!(result.is_ok());
        let mut paths = fake
            .calls()
            .into_iter()
            .map(|invocation| invocation.path)
            .collect::<Vec<_>>();
        paths.sort();
        assert_eq!(paths, vec![
            tmp.path().join("backend"),
            tmp.path().join("frontend")
        ]);
    }

    #[test]
    fn passes_repository_and_refspecs_through_to_git()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(
            ["fetch", "origin", "main"],
            0,
            "",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = run(&workspace, &cli_context(tmp.path()), &FetchArgs {
            repository: Some("origin".to_owned()),
            refspecs: vec!["main".to_owned()]
        });

        // Assert
        assert!(result.is_ok());
        assert_eq!(fake.calls().len(), 2);
    }

    use crate::git::RepoOutcome;
    use crate::test_support::repo;

    #[test]
    fn fetch_with_no_output_is_a_quiet_success()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(["fetch"], 0, "", ""));

        // Act
        let outcome = git
            .fetch(&repo("backend", "/vmr/backend"), &FetchArgs {
                repository: None,
                refspecs: vec![]
            })
            .unwrap();

        // Assert
        assert!(matches!(outcome, RepoOutcome::Success(None)));
    }

    #[test]
    fn fetch_success_reports_stderr_before_stdout()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["fetch"],
            0,
            "stdout chatter\n",
            "From origin\n"
        ));

        // Act
        let outcome = git
            .fetch(&repo("backend", "/vmr/backend"), &FetchArgs {
                repository: None,
                refspecs: vec![]
            })
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "From origin");
    }
}
