use crate::git::{
    Git, GitCommandResult, command_result, first_non_empty_line_with_fallback
};
use std::ffi::OsString;
use std::path::Path;

impl Git
{
    pub fn fetch(
        &self,
        repo_name: &str,
        repo_path: &Path,
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

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    ""
                );

                if message.is_empty() { None } else { Some(message) }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git fetch failed"
                )
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
    fn fetch_with_no_output_is_a_quiet_success()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(["fetch"], 0, "", ""));

        // Act
        let outcome =
            git.fetch("backend", Path::new("/vmr/backend"), None, &[]).unwrap();

        // Assert
        assert!(matches!(outcome, RepoOutcome::Success(None)));
    }
}
