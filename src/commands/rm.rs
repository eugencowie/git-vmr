use crate::cli::CliContext;
use crate::git::RmArgs;
use crate::render::Rendered;
use crate::workspace::{Scope, Workspace};
use anyhow::{Result, bail};

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: RmArgs
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    // Require explicit recursive intent for aggregate path removal
    if !args.options.recursive
        && args
            .paths
            .iter()
            .any(|path| workspace.is_aggregate(working_dir, path))
    {
        bail!("error: cannot remove VMR root without -r");
    }

    // Remove routed paths in each owning child repository
    workspace.run_routed(
        working_dir,
        Scope::Paths(args.paths.to_vec()),
        |git, repo, repo_paths| git.rm(repo, repo_paths, &args.options)
    )
}
