use crate::git::{self, ChmodMode, Git};
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn add(
    git: &Git,
    working_dir: &Path,
    paths: &[PathBuf],
    all: bool,
    force: bool,
    chmod: Option<ChmodMode>
) -> Result<()>
{
    // Find VMR root and route requested paths
    let vmr = Vmr::find(working_dir)?;
    let routing_working_dir =
        if paths.is_empty() && all { &vmr.path } else { working_dir };
    let paths = if paths.is_empty() && all
    {
        vec![PathBuf::from(".")]
    }
    else
    {
        paths.to_vec()
    };
    let routed = vmr
        .route_paths(routing_working_dir, &paths)?
        .into_iter()
        .collect::<Vec<_>>();

    // Stage paths in each child repository
    let results = routed
        .par_iter()
        .map(|(repo_path, repo_paths)| {
            git.add(
                &repo_path.name,
                &repo_path.path,
                repo_paths,
                all,
                force,
                chmod
            )
        })
        .collect::<Vec<_>>();

    git::print_results(results)
}
