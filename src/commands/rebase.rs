use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &RebaseArgs
) -> Result<Rendered>
{
    // Rebase in each child repository
    workspace.run(|git, repo| git.rebase(repo, args))
}

use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;

/// Reapply commits on top of another base tip
#[derive(clap::Args)]
pub struct RebaseArgs
{
    /// Upstream branch to compare against
    #[arg(required = true, value_name = "upstream")]
    pub upstream: String
}

impl Git
{
    fn rebase(&self, repo: &Repo, args: &RebaseArgs) -> GitCommandResult
    {
        let output =
            self.output(&repo.path, ["rebase", args.upstream.as_str()])?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrThenStdout, "git rebase failed")
        )
    }
}
