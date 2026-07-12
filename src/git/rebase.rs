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
    pub fn rebase(&self, repo: &Repo, args: &RebaseArgs) -> GitCommandResult
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
