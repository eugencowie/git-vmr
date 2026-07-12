use crate::cli::CliContext;
use crate::git::RestoreArgs;
use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::Result;

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: RestoreArgs
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    // Restore routed paths in each owning child repository
    workspace.run_routed(
        working_dir,
        Scope::Paths(args.paths.to_vec()),
        |git, repo, repo_paths| git.restore(repo, repo_paths, &args.options)
    )
}
