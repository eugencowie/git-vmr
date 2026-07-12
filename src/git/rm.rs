use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;
use std::path::PathBuf;

impl Git
{
    pub fn rm(
        &self,
        repo: &Repo,
        paths: &[PathBuf],
        recursive: bool,
        force: bool,
        dry_run: bool,
        cached: bool
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("rm")];

        if recursive
        {
            args.push(OsString::from("-r"));
        }

        if force
        {
            args.push(OsString::from("--force"));
        }

        if dry_run
        {
            args.push(OsString::from("--dry-run"));
        }

        if cached
        {
            args.push(OsString::from("--cached"));
        }

        args.push(OsString::from("--"));
        args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
        let output = self.output(&repo.path, args)?;

        let success = if dry_run
        {
            SuccessReport::line(Streams::StdoutOnly, OnEmpty::Quiet)
        }
        else
        {
            SuccessReport::quiet()
        };

        command_result(repo, &output, success, FailureReport::detailed("rm"))
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
    fn rm_reports_stdout_only_for_dry_runs()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["rm", "--dry-run", "--", "old.rs"],
            0,
            "rm 'old.rs'\n",
            ""
        ));

        // Act
        let outcome = git
            .rm(
                &repo("backend", "/vmr/backend"),
                &[PathBuf::from("old.rs")],
                false,
                false,
                true,
                false
            )
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "rm 'old.rs'");
    }
}
