use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &PushArgs
) -> Result<Rendered>
{
    // Push in each child repository
    workspace.run(|git, repo| git.push(repo, args))
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;

/// Update remote refs along with associated objects
#[derive(clap::Args)]
pub struct PushArgs
{
    /// The "remote" repository that is the destination of a push operation
    #[arg(value_name = "repository")]
    pub repository: Option<String>,

    /// Specify what destination ref to update with what source object
    #[arg(value_name = "refspec")]
    pub refspecs: Vec<String>
}

impl Git
{
    fn push(&self, repo: &Repo, push: &PushArgs) -> GitCommandResult
    {
        let mut args = vec![OsString::from("push")];

        if let Some(repository) = &push.repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(push.refspecs.iter().map(OsString::from));

        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::line(Streams::StderrThenStdout, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "git push failed")
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
    fn push_success_reports_stderr_before_stdout()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["push"],
            0,
            "stdout chatter\n",
            "main -> main\n"
        ));

        // Act
        let outcome = git
            .push(&repo("backend", "/vmr/backend"), &PushArgs {
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
        assert_eq!(message.message, "main -> main");
    }
}
