use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use std::path::Path;

impl Git
{
    pub fn merge(
        &self,
        repo_name: &str,
        repo_path: &Path,
        commit_ish: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["merge", commit_ish])?;

        command_result(
            repo_name,
            repo_path,
            &output,
            SuccessReport::line(
                Streams::StdoutOnly,
                OnEmpty::Text("git merge succeeded")
            ),
            FailureReport::line(Streams::StderrThenStdout, "git merge failed")
        )
    }
}
