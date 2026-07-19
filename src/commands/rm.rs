use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::{AggregatePolicy, Scope, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &RmArgs
) -> Result<Rendered>
{
    // Aggregate path removal requires explicit recursive intent
    let aggregate = if args.options.recursive
    {
        AggregatePolicy::Allow
    }
    else
    {
        AggregatePolicy::Deny(
            "error: cannot remove VMR root without -r".to_owned()
        )
    };

    // Remove routed paths in each owning child repository
    workspace.run_routed(
        Scope::Paths { paths: args.paths.to_vec(), aggregate },
        |git, repo, repo_paths| git.rm(repo, repo_paths, &args.options)
    )
}

use crate::git::report::{
    FailureReport, OnEmpty, Streams, SuccessReport, command_result
};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;
use std::path::PathBuf;

/// Remove files from the working tree and from the index
#[derive(clap::Args)]
pub struct RmArgs
{
    #[command(flatten)]
    pub options: RmOptions,

    /// Files to remove
    #[arg(required = true, num_args = 1.., value_name = "pathspec")]
    pub paths: Vec<PathBuf>
}

#[derive(clap::Args)]
pub struct RmOptions
{
    /// Allow recursive removal when a leading directory name is given
    #[arg(short)]
    pub recursive: bool,

    /// Override the up-to-date check
    #[arg(short, long)]
    pub force: bool,

    /// Don't actually remove any files
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Unstage and remove paths only from the index
    #[arg(long)]
    pub cached: bool
}

impl Git
{
    fn rm(
        &self,
        repo: &Repo,
        paths: &[PathBuf],
        options: &RmOptions
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("rm")];

        if options.recursive
        {
            args.push(OsString::from("-r"));
        }

        if options.force
        {
            args.push(OsString::from("--force"));
        }

        if options.dry_run
        {
            args.push(OsString::from("--dry-run"));
        }

        if options.cached
        {
            args.push(OsString::from("--cached"));
        }

        args.push(OsString::from("--"));
        let output = self.path_output(&repo.path, args, paths)?;

        let success = if options.dry_run
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
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};
    use crate::git::{RepoOutcome, ScriptedFake};
    use crate::test_support::repo;
    use std::path::PathBuf;

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
                &RmOptions {
                    recursive: false,
                    force: false,
                    dry_run: true,
                    cached: false
                }
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

    #[test]
    fn parses_rm_short_flags()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "rm",
            "-r",
            "-f",
            "-n",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Rm(RmArgs {
                options: RmOptions {
                    recursive: true,
                    force: true,
                    dry_run: true,
                    cached: false
                },
                paths
            })) if paths == [PathBuf::from("backend/src.rs")]
        ));
    }

    #[test]
    fn parses_rm_long_flags()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "rm",
            "--force",
            "--dry-run",
            "--cached",
            "backend/src.rs"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Rm(RmArgs {
                options: RmOptions {
                    recursive: false,
                    force: true,
                    dry_run: true,
                    cached: true
                },
                paths
            })) if paths == [PathBuf::from("backend/src.rs")]
        ));
    }
}
