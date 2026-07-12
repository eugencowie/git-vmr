use crate::cli::CliContext;
use crate::git::CommitArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::{Result, bail};

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
}
