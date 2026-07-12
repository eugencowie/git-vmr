use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;

impl Git
{
    pub fn merge(&self, repo: &Repo, commit_ish: &str) -> GitCommandResult
    {
        let output = self.output(&repo.path, ["merge", commit_ish])?;

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
