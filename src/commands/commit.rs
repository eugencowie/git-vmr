use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::{Context, Result, bail};

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: CommitArgs
) -> Result<Rendered>
{
    // Filter child repositories without staged changes
    let dirty_repos = workspace
        .map(|git, repo| Ok(git.is_dirty(&repo.path)?.then(|| repo.clone())))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Check if there are no dirty repositories
    if dirty_repos.is_empty()
    {
        bail!("error: nothing to commit, working tree clean");
    }

    // Commit in each dirty repository
    workspace.run_in(&dirty_repos, |git, repo| git.commit(repo, &args))
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult, stderr};
use crate::vmr::Repo;
use std::path::Path;

/// Record changes to the repositories
#[derive(clap::Args)]
pub struct CommitArgs
{
    /// Use <msg> as the commit message
    #[arg(short, long, required = true, value_name = "msg")]
    pub message: String
}

impl Git
{
    fn commit(&self, repo: &Repo, args: &CommitArgs) -> GitCommandResult
    {
        let output =
            self.output(&repo.path, ["commit", "-m", &args.message])?;

        command_result(
            repo,
            &output,
            SuccessReport::line(
                Streams::StdoutOnly,
                OnEmpty::Text("git commit succeeded")
            ),
            FailureReport::line(Streams::StderrOnly, "git commit failed")
        )
    }

    fn is_dirty(&self, repo_path: &Path) -> Result<bool>
    {
        let output = self
            .output(repo_path, ["diff", "--cached", "--quiet"])
            .with_context(|| {
                format!(
                    "fatal: failed to inspect staged changes for '{}'",
                    repo_path.display()
                )
            })?;

        match output.status.code()
        {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => bail!(
                "fatal: git diff --cached --quiet failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            )
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, ScriptedFake};
    use crate::test_support::{cli_context, vmr_fixture};
    use std::ffi::OsString;
    use std::sync::Arc;

    #[test]
    fn bails_when_no_child_repo_has_staged_changes()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake =
            ScriptedFake::new().on(["diff", "--cached", "--quiet"], 0, "", "");
        let git = Git::with(fake);
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let err = run(&workspace, &cli_context(tmp.path()), CommitArgs {
            message: "message".to_owned()
        })
        .unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: nothing to commit, working tree clean"
        );
    }

    #[test]
    fn commits_in_dirty_child_repos()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["diff", "--cached", "--quiet"], 1, "", "")
                .on(["commit", "-m", "message"], 0, "committed", "")
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = run(&workspace, &cli_context(tmp.path()), CommitArgs {
            message: "message".to_owned()
        });

        // Assert
        assert!(result.is_ok());
        let mut commits = fake
            .calls()
            .into_iter()
            .filter(|invocation| {
                invocation.args.first() == Some(&OsString::from("commit"))
            })
            .map(|invocation| invocation.path)
            .collect::<Vec<_>>();
        commits.sort();
        assert_eq!(commits, vec![
            tmp.path().join("backend"),
            tmp.path().join("frontend")
        ]);
    }

    use crate::git::RepoOutcome;
    use crate::test_support::repo;

    #[test]
    fn is_dirty_maps_diff_exit_codes()
    {
        for (code, expected) in [(0, false), (1, true)]
        {
            // Arrange
            let git = Git::with(ScriptedFake::new().on(
                ["diff", "--cached", "--quiet"],
                code,
                "",
                ""
            ));

            // Act
            let dirty = git.is_dirty(Path::new("/vmr/backend")).unwrap();

            // Assert
            assert_eq!(dirty, expected);
        }
    }

    #[test]
    fn commit_reports_first_stdout_line_through_the_seam()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["commit", "-m", "message"],
            0,
            "[main abc1234] message\n 1 file changed\n",
            ""
        ));

        // Act
        let outcome = git
            .commit(&repo("backend", "/vmr/backend"), &CommitArgs {
                message: "message".to_owned()
            })
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.repo, "backend");
        assert_eq!(message.message, "[main abc1234] message");
    }

    #[test]
    fn commit_reports_first_stderr_line_on_failure()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["commit", "-m", "message"],
            1,
            "",
            "error: nothing to commit\n"
        ));

        // Act
        let outcome = git
            .commit(&repo("backend", "/vmr/backend"), &CommitArgs {
                message: "message".to_owned()
            })
            .unwrap();

        // Assert
        let RepoOutcome::Failure(message) = outcome
        else
        {
            panic!("expected failure");
        };
        assert_eq!(message.message, "error: nothing to commit");
    }
}
