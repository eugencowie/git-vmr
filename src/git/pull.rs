use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use std::ffi::OsString;
use std::path::Path;

impl Git
{
    pub fn pull(
        &self,
        repo_name: &str,
        repo_path: &Path,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("pull")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            repo_path,
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
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;

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
        let outcome =
            git.pull("backend", Path::new("/vmr/backend"), None, &[]).unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "Already up to date.");
    }
}
