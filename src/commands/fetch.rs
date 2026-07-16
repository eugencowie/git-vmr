use crate::cli::CliContext;
use crate::git::FetchArgs;
use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: FetchArgs
) -> Result<Rendered>
{
    // Fetch in each child repository
    workspace.run(|git, repo| git.fetch(repo, &args))
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, ScriptedFake};
    use crate::test_support::{cli_context, vmr_fixture};
    use std::sync::Arc;

    #[test]
    fn fetches_in_every_child_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(["fetch"], 0, "", ""));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = run(&workspace, &cli_context(tmp.path()), FetchArgs {
            repository: None,
            refspecs: vec![]
        });

        // Assert
        assert!(result.is_ok());
        let mut paths = fake
            .calls()
            .into_iter()
            .map(|invocation| invocation.path)
            .collect::<Vec<_>>();
        paths.sort();
        assert_eq!(paths, vec![
            tmp.path().join("backend"),
            tmp.path().join("frontend")
        ]);
    }

    #[test]
    fn passes_repository_and_refspecs_through_to_git()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(
            ["fetch", "origin", "main"],
            0,
            "",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = run(&workspace, &cli_context(tmp.path()), FetchArgs {
            repository: Some("origin".to_owned()),
            refspecs: vec!["main".to_owned()]
        });

        // Assert
        assert!(result.is_ok());
        assert_eq!(fake.calls().len(), 2);
    }
}
