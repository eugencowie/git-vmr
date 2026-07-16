use crate::cli::CliContext;
use crate::git::AddArgs;
use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: AddArgs
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    // `add -A` with no paths stages the entire VMR
    let scope = if args.paths.is_empty() && args.options.all
    {
        Scope::EntireVmr
    }
    else
    {
        Scope::Paths(args.paths.to_vec())
    };

    // Stage routed paths in each owning child repository
    workspace.run_routed(working_dir, scope, |git, repo, repo_paths| {
        git.add(repo, repo_paths, &args.options)
    })
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{AddOptions, Git, ScriptedFake};
    use crate::test_support::{cli_context, vmr_fixture};
    use std::path::PathBuf;
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
        let result = run(&workspace, &cli_context(tmp.path()), AddArgs {
            options: AddOptions { all: true, force: false, chmod: None },
            paths: vec![]
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
        let result = run(&workspace, &cli_context(tmp.path()), AddArgs {
            options: AddOptions { all: false, force: false, chmod: None },
            paths
        });

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
