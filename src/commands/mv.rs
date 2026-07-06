use crate::git::Git;
use crate::vmr::{Repo, Vmr};
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn mv(
    git: &Git,
    working_dir: &Path,
    sources: &[PathBuf],
    destination: &Path
) -> Result<()>
{
    // Find VMR root and route all operands before moving anything
    let vmr = Vmr::find(working_dir)?;
    let sources = sources
        .iter()
        .map(|source| vmr.route_single_path(working_dir, source))
        .collect::<Result<Vec<_>>>()?;
    let destination = vmr.route_single_path(working_dir, destination)?;

    if sources.len() == 1 && sources[0].0 == destination.0
    {
        git.mv(&sources[0].0.path, &sources[0].1, &destination.1)
    }
    else
    {
        let plan = MovePlan::build(git, sources, destination)?;
        execute_plan(git, plan)
    }
}

struct MovePlan
{
    entries: Vec<MovePlanEntry>,
    destination_repo: Repo,
    destination_relative: PathBuf,
    multi_source: bool
}

struct MovePlanEntry
{
    source_repo: Repo,
    source_relative: PathBuf,
    destination_repo: Repo,
    final_destination_relative: PathBuf
}

impl MovePlan
{
    fn build(
        git: &Git,
        sources: Vec<(Repo, PathBuf)>,
        destination: (Repo, PathBuf)
    ) -> Result<Self>
    {
        let (destination_repo, destination_relative) = destination;
        let multi_source = sources.len() > 1;
        let mut final_destinations = HashSet::new();
        let mut entries = Vec::new();

        if multi_source
        {
            let destination_path =
                destination_repo.path.join(&destination_relative);
            if !destination_path.is_dir()
            {
                bail!(
                    "error: destination '{}' is not an existing directory",
                    destination_path.display()
                );
            }
        }

        for (source_repo, source_relative) in sources
        {
            git.ensure_tracked(&source_repo.path, &source_relative)?;

            let final_destination_relative = if multi_source
            {
                let source_name = source_relative
                    .file_name()
                    .context("error: source path does not have a file name")?;
                destination_relative.join(source_name)
            }
            else
            {
                final_destination_path(
                    &source_relative,
                    &destination_repo.path,
                    &destination_relative
                )?
            };

            let destination_key =
                destination_repo.path.join(&final_destination_relative);
            if !final_destinations.insert(destination_key.clone())
            {
                bail!(
                    "error: duplicate destination '{}'",
                    destination_key.display()
                );
            }
            if destination_key.exists()
            {
                bail!(
                    "error: destination '{}' already exists",
                    destination_key.display()
                );
            }

            entries.push(MovePlanEntry {
                source_repo,
                source_relative,
                destination_repo: destination_repo.clone(),
                final_destination_relative
            });
        }

        Ok(Self {
            entries,
            destination_repo,
            destination_relative,
            multi_source
        })
    }
}

fn execute_plan(git: &Git, plan: MovePlan) -> Result<()>
{
    if plan.multi_source
        && plan
            .entries
            .iter()
            .all(|entry| entry.source_repo == plan.destination_repo)
    {
        let sources = plan
            .entries
            .iter()
            .map(|entry| entry.source_relative.clone())
            .collect::<Vec<_>>();
        return git.mv_to_directory(
            &plan.destination_repo.path,
            &sources,
            &plan.destination_relative
        );
    }

    for entry in plan.entries
    {
        mv_between_repos(git, entry)?;
    }

    Ok(())
}

fn mv_between_repos(git: &Git, entry: MovePlanEntry) -> Result<()>
{
    let source_path = entry.source_repo.path.join(&entry.source_relative);
    let destination_path =
        entry.destination_repo.path.join(&entry.final_destination_relative);

    // Move the worktree path across child repositories
    fs::rename(&source_path, &destination_path).with_context(|| {
        format!(
            "fatal: failed to move '{}' to '{}'",
            source_path.display(),
            destination_path.display()
        )
    })?;

    // Stage the source deletion and destination addition in their repositories
    git.add_path(&entry.source_repo.path, &entry.source_relative)
        .with_context(|| {
            format!(
                "fatal: failed to stage source deletion in '{}'",
                entry.source_repo.path.display()
            )
        })?;
    git.add_path(
        &entry.destination_repo.path,
        &entry.final_destination_relative
    )
    .with_context(|| {
        format!(
            "fatal: failed to stage destination addition in '{}'",
            entry.destination_repo.path.display()
        )
    })?;

    Ok(())
}

fn final_destination_path(
    source_relative: &Path,
    destination_repo: &Path,
    destination_relative: &Path
) -> Result<PathBuf>
{
    let destination_path = destination_repo.join(destination_relative);

    // Treat an existing destination directory like git mv does
    if destination_path.is_dir()
    {
        let source_name = source_relative
            .file_name()
            .context("error: source path does not have a file name")?;
        return Ok(destination_relative.join(source_name));
    }

    // Reject conflicts and missing parents before moving the source
    if destination_path.exists()
    {
        bail!(
            "error: destination '{}' already exists",
            destination_path.display()
        );
    }

    let parent = destination_path
        .parent()
        .context("error: destination path does not have a parent")?;
    if !parent.is_dir()
    {
        bail!(
            "error: destination parent '{}' does not exist",
            parent.display()
        );
    }

    Ok(destination_relative.to_path_buf())
}
