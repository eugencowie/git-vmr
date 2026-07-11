use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use std::ffi::OsString;
use std::path::Path;

impl Git
{
    pub fn push(
        &self,
        repo_name: &str,
        repo_path: &Path,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("push")];

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
            SuccessReport::Line {
                from: Streams::StderrThenStdout,
                on_empty: OnEmpty::Quiet
            },
            FailureReport::Line {
                from: Streams::StderrThenStdout,
                fallback: "git push failed"
            }
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
        let outcome =
            git.push("backend", Path::new("/vmr/backend"), None, &[]).unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "main -> main");
    }
}
