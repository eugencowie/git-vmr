use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: PullArgs
) -> Result<Rendered>
{
    // Pull in each child repository
    workspace.run(|git, repo| git.pull(repo, &args))
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;

/// Fetch from and integrate with another repository or a local branch
#[derive(clap::Args)]
pub struct PullArgs
{
    /// The "remote" repository to pull from
    #[arg(value_name = "repository")]
    pub repository: Option<String>,

    /// Which branch or other reference(s) to fetch and integrate into the
    /// current branch
    #[arg(value_name = "refspec")]
    pub refspecs: Vec<String>
}

impl Git
{
    fn pull(&self, repo: &Repo, pull: &PullArgs) -> GitCommandResult
    {
        let mut args = vec![OsString::from("pull")];

        if let Some(repository) = &pull.repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(pull.refspecs.iter().map(OsString::from));

        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::line(Streams::StdoutThenStderr, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "git pull failed")
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
    fn pull_success_reports_stdout_before_stderr()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["pull"],
            0,
            "Already up to date.\n",
            "stderr chatter\n"
        ));

        // Act
        let outcome = git
            .pull(&repo("backend", "/vmr/backend"), &PullArgs {
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
        assert_eq!(message.message, "Already up to date.");
    }
}
