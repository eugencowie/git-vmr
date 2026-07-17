use crate::cli::CliContext;
use crate::render::Rendered;
use crate::workspace::{AggregatePolicy, Scope, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    _context: &CliContext,
    args: &RestoreArgs
) -> Result<Rendered>
{
    // The aggregate path restores every child repo, matching `git restore`
    let scope = Scope::Paths {
        paths: args.paths.to_vec(),
        aggregate: AggregatePolicy::Allow
    };

    // Restore routed paths in each owning child repository
    workspace.run_routed(scope, |git, repo, repo_paths| {
        git.restore(repo, repo_paths, &args.options)
    })
}

use crate::git::report::{FailureReport, SuccessReport, command_result};
use crate::git::{Git, GitCommandResult};
use crate::vmr::Repo;
use std::ffi::OsString;
use std::path::PathBuf;

/// Restore working tree files
#[derive(clap::Args)]
pub struct RestoreArgs
{
    #[command(flatten)]
    pub options: RestoreOptions,

    /// Files to restore
    #[arg(required = true, num_args = 1.., value_name = "pathspec")]
    pub paths: Vec<PathBuf>
}

#[derive(clap::Args)]
pub struct RestoreOptions
{
    /// Restore the working tree
    #[arg(long)]
    pub worktree: bool,

    /// Restore the index
    #[arg(long)]
    pub staged: bool
}

impl Git
{
    fn restore(
        &self,
        repo: &Repo,
        paths: &[PathBuf],
        options: &RestoreOptions
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("restore")];

        if options.staged
        {
            args.push(OsString::from("--staged"));
        }

        if options.worktree
        {
            args.push(OsString::from("--worktree"));
        }

        args.push(OsString::from("--"));
        args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
        let output = self.output(&repo.path, args)?;

        command_result(
            repo,
            &output,
            SuccessReport::quiet(),
            FailureReport::detailed("restore")
        )
    }
}
