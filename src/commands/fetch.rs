use crate::render::Rendered;
use crate::workspace::Workspace;
use anyhow::Result;

pub fn fetch(
    workspace: &Workspace,
    repository: Option<&str>,
    refspecs: &[String]
) -> Result<Rendered>
{
    // Fetch in each child repository
    workspace.run(|git, repo| {
        git.fetch(&repo.name, &repo.path, repository, refspecs)
    })
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, ScriptedFake};
    use std::fs;
    use std::sync::Arc;

    fn vmr_fixture() -> tempfile::TempDir
    {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
        fs::create_dir_all(tmp.path().join("frontend/.git")).unwrap();
        tmp
    }

    #[test]
    fn fetches_in_every_child_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(["fetch"], 0, "", ""));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = fetch(&workspace, None, &[]);

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
        let result = fetch(&workspace, Some("origin"), &["main".to_owned()]);

        // Assert
        assert!(result.is_ok());
        assert_eq!(fake.calls().len(), 2);
    }
}
