use crate::git::{
    Git, GitCommandResult, command_result, first_non_empty_line, stderr
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl Git
{
    #[expect(clippy::too_many_arguments, reason = "mirrors git rm flags")]
    pub fn rm(
        &self,
        repo_name: &str,
        repo_path: &Path,
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
        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                if dry_run
                {
                    let message = first_non_empty_line(&output.stdout, "");

                    if message.is_empty() { None } else { Some(message) }
                }
                else
                {
                    None
                }
            },
            |output| {
                format!(
                    "git rm failed for '{}': {}",
                    repo_path.display(),
                    stderr(output)
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
                "backend",
                Path::new("/vmr/backend"),
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
