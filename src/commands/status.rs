use crate::cli::CliContext;
use crate::render::{self, Rendered};
use crate::workspace::Workspace;
use anyhow::Result;

pub fn run(workspace: &Workspace, context: &CliContext) -> Result<Rendered>
{
    let display_name = &context.display_name;
    let working_dir = &context.working_dir;

    // Collect status information from child repositories, in repo order
    let statuses = workspace
        .map(|git, repo| git.status(repo))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // Render status output
    Ok(render::status(&statuses, display_name, working_dir).into())
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::cli_context;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    const DISPLAY_NAME: &str = "git vmr";

    #[test]
    fn status_succeeds_with_git_repos_and_non_git_dirs()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir(tmp.path().join("docs")).unwrap();
        init_git_repo(&tmp.path().join("backend"));

        // Act
        let git = crate::git::Git::subprocess();
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let mut context = cli_context(tmp.path());
        context.display_name = DISPLAY_NAME.to_owned();
        let result = run(&workspace, &context);

        // Assert
        assert!(result.is_ok());
    }

    fn init_git_repo(path: &Path)
    {
        fs::create_dir(path).unwrap();

        Command::new("git").arg("init").current_dir(path).output().unwrap();
    }
}
