use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;

impl Git
{
    pub fn fetch(
        &self,
        repo: &Repo,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("fetch")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::line(Streams::StderrThenStdout, OnEmpty::Quiet),
            FailureReport::line(Streams::StderrThenStdout, "git fetch failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;
    use crate::test_support::repo;

    #[test]
    fn fetch_with_no_output_is_a_quiet_success()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(["fetch"], 0, "", ""));

        // Act
        let outcome =
            git.fetch(&repo("backend", "/vmr/backend"), None, &[]).unwrap();

        // Assert
        assert!(matches!(outcome, RepoOutcome::Success(None)));
    }

    #[test]
    fn fetch_success_reports_stderr_before_stdout()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["fetch"],
            0,
            "stdout chatter\n",
            "From origin\n"
        ));

        // Act
        let outcome =
            git.fetch(&repo("backend", "/vmr/backend"), None, &[]).unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "From origin");
    }
}
