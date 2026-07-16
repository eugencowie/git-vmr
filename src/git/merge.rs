use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;

/// Join two or more development histories together
#[derive(clap::Args)]
pub struct MergeArgs
{
    /// Commits, usually other branch heads, to merge into our branch
    #[arg(required = true, value_name = "commit")]
    pub commit_ish: String
}

impl Git
{
    pub fn merge(&self, repo: &Repo, args: &MergeArgs) -> GitCommandResult
    {
        let output =
            self.output(&repo.path, ["merge", args.commit_ish.as_str()])?;

        command_result(
            repo,
            &output,
            SuccessReport::line(
                Streams::StdoutOnly,
                OnEmpty::Text("git merge succeeded")
            ),
            FailureReport::line(Streams::StderrThenStdout, "git merge failed")
        )
    }
}
