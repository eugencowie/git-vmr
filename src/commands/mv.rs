use crate::git::{Git, GitCommandResult, failure_message, quiet_success};
use crate::render::{self, Rendered};
use crate::workspace::{Repo, Workspace};
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn mv(
    workspace: &Workspace,
    working_dir: &Path,
    sources: &[PathBuf],
    destination: &Path
) -> Result<Rendered>
{
    // Route all operands before moving anything
    let git = workspace.git();
    let sources = sources
        .iter()
        .map(|source| workspace.route_single(working_dir, source))
        .collect::<Result<Vec<_>>>()?;
    let destination = workspace.route_single(working_dir, destination)?;

    let results = if sources.len() == 1 && sources[0].0 == destination.0
    {
        let (repo, source_relative) = &sources[0];
        vec![in_repo_outcome(
            repo,
            git.mv(&repo.path, source_relative, &destination.1)
        )]
    }
    else
    {
        let plan = MovePlan::build(git, sources, destination)?;
        execute_plan(git, plan)
    };

    render::outcomes(results)
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

/// Attempts every entry of the plan, producing one repo outcome per entry so
/// result aggregation reports exactly which moves landed.
fn execute_plan(git: &Git, plan: MovePlan) -> Vec<GitCommandResult>
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
        return vec![in_repo_outcome(
            &plan.destination_repo,
            git.mv_to_directory(
                &plan.destination_repo.path,
                &sources,
                &plan.destination_relative
            )
        )];
    }

    plan.entries.into_iter().map(|entry| mv_between_repos(git, entry)).collect()
}

fn in_repo_outcome(repo: &Repo, result: Result<()>) -> GitCommandResult
{
    Ok(match result
    {
        Ok(()) => quiet_success(),
        Err(error) => failure_message(&repo.name, format!("{error:#}"))
    })
}

fn mv_between_repos(git: &Git, entry: MovePlanEntry) -> GitCommandResult
{
    let source_path = entry.source_repo.path.join(&entry.source_relative);
    let destination_path =
        entry.destination_repo.path.join(&entry.final_destination_relative);

    // Move the worktree path across child repositories
    if let Err(error) = fs::rename(&source_path, &destination_path)
    {
        return Ok(failure_message(
            &entry.source_repo.name,
            format!(
                "fatal: failed to move '{}' to '{}': {error}",
                source_path.display(),
                destination_path.display()
            )
        ));
    }

    // Stage the source deletion and destination addition in their repositories
    if let Err(error) =
        git.add_path(&entry.source_repo.path, &entry.source_relative)
    {
        return Ok(failure_message(
            &entry.source_repo.name,
            format!(
                "fatal: failed to stage source deletion in '{}': {error:#}",
                entry.source_repo.path.display()
            )
        ));
    }

    if let Err(error) = git.add_path(
        &entry.destination_repo.path,
        &entry.final_destination_relative
    )
    {
        return Ok(failure_message(
            &entry.source_repo.name,
            format!(
                "fatal: failed to stage destination addition in '{}': {error:#}",
                entry.destination_repo.path.display()
            )
        ));
    }

    Ok(quiet_success())
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;
    use crate::render::Failed;
    use crate::test_support::vmr_fixture;
    use std::sync::Arc;

    /// A tracked file in a child repo: the file exists on disk and the fake
    /// answers the tracking probe for it.
    fn tracked_file(
        tmp: &tempfile::TempDir,
        fake: ScriptedFake,
        repo: &str,
        relative: &str
    ) -> ScriptedFake
    {
        let path = tmp.path().join(repo).join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "content\n").unwrap();
        fake.on(["ls-files", "--error-unmatch", "--", relative], 0, "", "")
    }

    #[test]
    fn cross_repo_move_renames_and_stages_both_sides()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new()
            .on(["add", "--", "config.toml"], 0, "", "")
            .on(["add", "--", "settings.toml"], 0, "", "");
        let fake = Arc::new(tracked_file(&tmp, fake, "backend", "config.toml"));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = mv(
            &workspace,
            tmp.path(),
            &[PathBuf::from("backend/config.toml")],
            Path::new("frontend/settings.toml")
        );

        // Assert: the file moved on disk and both repos staged their side
        assert!(result.unwrap().stdout.is_empty());
        assert!(!tmp.path().join("backend/config.toml").exists());
        assert!(tmp.path().join("frontend/settings.toml").exists());
        let staged = fake
            .calls()
            .into_iter()
            .filter(|invocation| {
                invocation.args.first().map(|arg| arg.to_string_lossy())
                    == Some("add".into())
            })
            .map(|invocation| invocation.path)
            .collect::<Vec<_>>();
        assert_eq!(staged, vec![
            tmp.path().join("backend"),
            tmp.path().join("frontend")
        ]);
    }

    #[test]
    fn planning_rejects_duplicate_destinations_before_moving_anything()
    {
        // Arrange: two sources in different repos collapse onto one name
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new();
        let fake = tracked_file(&tmp, fake, "backend", "notes.md");
        let fake = Arc::new(tracked_file(&tmp, fake, "frontend", "notes.md"));
        fs::create_dir(tmp.path().join("backend/docs")).unwrap();
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = mv(
            &workspace,
            tmp.path(),
            &[
                PathBuf::from("backend/notes.md"),
                PathBuf::from("frontend/notes.md")
            ],
            Path::new("backend/docs")
        );

        // Assert: planning fails and no source has moved
        let error = result.unwrap_err().to_string();
        assert!(error.contains("duplicate destination"));
        assert!(tmp.path().join("backend/notes.md").exists());
        assert!(tmp.path().join("frontend/notes.md").exists());
    }

    #[test]
    fn planning_rejects_multi_source_move_to_a_non_directory()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new();
        let fake = tracked_file(&tmp, fake, "backend", "a.txt");
        let fake = Arc::new(tracked_file(&tmp, fake, "backend", "b.txt"));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = mv(
            &workspace,
            tmp.path(),
            &[PathBuf::from("backend/a.txt"), PathBuf::from("backend/b.txt")],
            Path::new("frontend/missing")
        );

        // Assert
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("is not an existing directory")
        );
    }

    #[test]
    fn failed_staging_reports_the_entry_and_still_attempts_the_rest()
    {
        // Arrange: staging fails in backend, succeeds everywhere else
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new()
            .on(["add", "--", "a.txt"], 1, "", "fatal: unable to stage")
            .on(["add", "--", "b.txt"], 0, "", "")
            .on(["add", "--", "docs/a.txt"], 0, "", "")
            .on(["add", "--", "docs/b.txt"], 0, "", "");
        let fake = tracked_file(&tmp, fake, "backend", "a.txt");
        let fake = Arc::new(tracked_file(&tmp, fake, "backend", "b.txt"));
        fs::create_dir(tmp.path().join("frontend/docs")).unwrap();
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = mv(
            &workspace,
            tmp.path(),
            &[PathBuf::from("backend/a.txt"), PathBuf::from("backend/b.txt")],
            Path::new("frontend/docs")
        );

        // Assert: the failure names its entry and the other entry landed
        let failed = result.unwrap_err().downcast::<Failed>().unwrap();
        assert!(failed.message.contains("failed to stage source deletion"));
        assert!(failed.message.contains("backend"));
        assert!(tmp.path().join("frontend/docs/a.txt").exists());
        assert!(tmp.path().join("frontend/docs/b.txt").exists());
    }
}
