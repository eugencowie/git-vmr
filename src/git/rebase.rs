use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;

impl Git
{
    pub fn rebase(&self, repo: &Repo, upstream: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["rebase", upstream])?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::line(Streams::StderrThenStdout, "git rebase failed")
        )
    }
}
