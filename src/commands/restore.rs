use crate::git;
use crate::git::Git;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn restore(
    git: &Git,
    working_dir: &Path,
    paths: &[PathBuf],
    worktree: bool,
    staged: bool
) -> Result<()>
{
    // Find VMR root and route all requested paths before mutating any
    // repository
    let vmr = Vmr::find(working_dir)?;
    let routed =
        vmr.route_paths(working_dir, paths)?.into_iter().collect::<Vec<_>>();

    // Restore paths in each child repository
    let results = routed
        .par_iter()
        .map(|(repo_path, repo_paths)| {
            git.restore(
                &repo_path.name,
                &repo_path.path,
                repo_paths,
                worktree,
                staged
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)
}
