use crate::cli::CliContext;
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

use crate::git::report::{FailureReport, SuccessReport, command_result};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChmodMode
{
    Executable,
    NotExecutable
}

impl ChmodMode
{
    fn as_git_value(self) -> &'static str
    {
        match self
        {
            ChmodMode::Executable => "+x",
            ChmodMode::NotExecutable => "-x"
        }
    }
}

impl Display for ChmodMode
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(self.as_git_value())
    }
}

impl FromStr for ChmodMode
{
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err>
    {
        match value
        {
            "+x" => Ok(ChmodMode::Executable),
            "-x" => Ok(ChmodMode::NotExecutable),
            _ => Err("expected +x or -x".to_owned())
        }
    }
}

/// Add file contents to the index
#[derive(clap::Args)]
pub struct AddArgs
{
    #[command(flatten)]
    pub options: AddOptions,

    /// Files to add content from
    #[arg(required_unless_present = "all", num_args = 0.., value_name = "pathspec")]
    pub paths: Vec<PathBuf>
}

#[derive(clap::Args)]
pub struct AddOptions
{
    /// Allow adding otherwise ignored files
    #[arg(short, long)]
    pub force: bool,

    /// Update the index not only where the working tree has a file
    /// matching [pathspec] but also where the index already has an
    /// entry
    #[arg(short = 'A', long)]
    pub all: bool,

    /// Override the executable bit of added files
    #[arg(long, value_name = "(+|-)x")]
    pub chmod: Option<ChmodMode>
}

fn add_args(options: &AddOptions) -> Vec<OsString>
{
    let mut args = vec![OsString::from("add")];

    if options.all
    {
        args.push(OsString::from("--all"));
    }

    if options.force
    {
        args.push(OsString::from("--force"));
    }

    if let Some(chmod) = options.chmod
    {
        args.push(OsString::from(format!("--chmod={chmod}")));
    }

    args.push(OsString::from("--"));
    args
}

impl Git
{
    fn add(
        &self,
        repo: &Repo,
        paths: &[PathBuf],
        options: &AddOptions
    ) -> GitCommandResult
    {
        let output = self.path_output(&repo.path, add_args(options), paths)?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::detailed("add")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};
    use crate::git::{Git, ScriptedFake};
    use crate::test_support::{cli_context, vmr_fixture};
    use clap::error::ErrorKind;
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

    #[test]
    fn parses_add_flags()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "add",
            "-A",
            "-f",
            "--chmod=+x",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Add(AddArgs {
                options: AddOptions { all: true, force: true, chmod: Some(ChmodMode::Executable) },
                paths
            })) if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_add_long_flags()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "add",
            "--all",
            "--force",
            "--chmod=-x",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Add(AddArgs {
                options: AddOptions { all: true, force: true, chmod: Some(ChmodMode::NotExecutable) },
                paths
            })) if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_add_all_without_pathspecs()
    {
        // Act
        let cli = Cli::parse_from(["git-vmr", "add", "-A"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Add(AddArgs {
                options: AddOptions { all: true, force: false, chmod: None },
                paths
            })) if paths.is_empty()
        ));
    }

    #[test]
    fn rejects_add_without_pathspecs_or_all()
    {
        // Act
        let err = match Cli::parse_from(["git-vmr", "add"])
        {
            Ok(_) => panic!("expected add without pathspecs or all to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_add_invalid_chmod()
    {
        // Act
        let err = match Cli::parse_from([
            "git-vmr",
            "add",
            "--chmod=bad",
            "backend/src.rs"
        ])
        {
            Ok(_) => panic!("expected invalid chmod to fail"),
            Err(err) => err
        };

        // Assert
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }
}
