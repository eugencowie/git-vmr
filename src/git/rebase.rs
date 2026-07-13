use crate::git::report::{
    FailureReport, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use std::path::Path;

impl Git
{
    pub fn rebase(
        &self,
        repo_name: &str,
        repo_path: &Path,
        upstream: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["rebase", upstream])?;

        command_result(
            repo_name,
            repo_path,
            &output,
            SuccessReport::Quiet,
            FailureReport::Line {
                from: Streams::StderrThenStdout,
                fallback: "git rebase failed"
            }
        )
    }
}
