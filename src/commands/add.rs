use crate::git::ChmodMode;
use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn add(
    workspace: &Workspace,
    working_dir: &Path,
    paths: &[PathBuf],
    all: bool,
    force: bool,
    chmod: Option<ChmodMode>
) -> Result<Rendered>
{
    // `add -A` with no paths stages the entire VMR
    let scope = if paths.is_empty() && all
    {
        Scope::EntireVmr
    }
    else
    {
        Scope::Paths(paths.to_vec())
    };

    // Stage routed paths in each owning child repository
    workspace.run_routed(working_dir, scope, |git, repo, repo_paths| {
        git.add(repo, repo_paths, all, force, chmod)
    })
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, ScriptedFake};
    use crate::test_support::vmr_fixture;
    use std::sync::Arc;

    #[test]
    fn add_all_without_paths_stages_the_entire_vmr()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(
            ["add", "--all", "--", "."],
            0,
            "",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = add(&workspace, tmp.path(), &[], true, false, None);

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
    fn explicit_paths_only_stage_in_the_owning_child_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(
            ["add", "--", "src/main.rs"],
            0,
            "",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let paths = vec![PathBuf::from("backend/src/main.rs")];

        // Act
        let result = add(&workspace, tmp.path(), &paths, false, false, None);

        // Assert
        assert!(result.is_ok());
        assert_eq!(
            fake.calls()
                .into_iter()
                .map(|invocation| invocation.path)
                .collect::<Vec<_>>(),
            vec![tmp.path().join("backend")]
        );
    }
}
