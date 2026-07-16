use crate::cli::CliContext;
use crate::git::{
    Git, GitCommandResult, failure_message, quiet_success, stderr
};
use crate::render::{self, Rendered};
use crate::workspace::{Repo, Workspace};
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Move or rename a file, a directory, or a symlink
#[derive(clap::Args)]
pub struct MvArgs
{
    /// Files to move
    #[arg(required = true, num_args = 1.., value_name = "source")]
    pub sources: Vec<PathBuf>,

    /// Destination path
    #[arg(value_name = "destination")]
    pub destination: PathBuf
}

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: &MvArgs
) -> Result<Rendered>
{
    let moved = mv(workspace, &context.working_dir, args)?;
    render::outcomes(
        moved.outcomes,
        moved.scope_repos.iter().map(String::as_str)
    )
}

/// The result of a move: one outcome per executed move entry, plus the
/// names of the child repos in the command's routed scope — every source's
/// repo and the destination's, duplicates included. Entries and scope can
/// differ when several entries move between the same repos.
struct MoveOutcomes
{
    outcomes: Vec<GitCommandResult>,
    scope_repos: Vec<String>
}

/// Moves sources to a destination across the workspace: routes every
/// operand, builds the move plan in full, then executes it entry by
/// entry, producing one repo outcome per entry.
///
/// Execution is deliberately serial: entries sharing a destination repo
/// would otherwise race concurrent `git add` invocations on one index.
/// When a cross-repo rename lands but staging fails, the entry reports a
/// failure and the half-done state stays: the file remains moved on
/// disk, with the deletion and addition left unstaged.
fn mv(
    workspace: &Workspace,
    working_dir: &Path,
    args: &MvArgs
) -> Result<MoveOutcomes>
{
    // Route all operands before moving anything
    let git = workspace.git();
    let sources = args
        .sources
        .iter()
        .map(|source| workspace.route_single(working_dir, source))
        .collect::<Result<Vec<_>>>()?;
    let destination = workspace.route_single(working_dir, &args.destination)?;
    let scope_repos = sources
        .iter()
        .map(|(repo, _)| repo.name.clone())
        .chain(std::iter::once(destination.0.name.clone()))
        .collect::<Vec<_>>();

    if sources.len() == 1 && sources[0].0 == destination.0
    {
        let (repo, source_relative) = &sources[0];
        return Ok(MoveOutcomes {
            outcomes: vec![in_repo_outcome(
                repo,
                git.mv(&repo.path, source_relative, &destination.1)
            )],
            scope_repos
        });
    }

    let plan = MovePlan::build(git, sources, destination)?;
    Ok(MoveOutcomes { outcomes: execute_plan(git, plan), scope_repos })
}

impl Git
{
    /// Stage one path, plainly: move-plan execution stages each landed file
    /// in its destination repo, outside any report policy.
    fn add_path(&self, repo_path: &Path, path: &Path) -> Result<()>
    {
        let output = self.path_output(repo_path, ["add", "--"], [path])?;

        if !output.status.success()
        {
            bail!(
                "git add failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
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

/// Executes a same-repository multi-source plan as one batched `git mv` and
/// produces one aggregated repo outcome. Other plans produce one outcome per
/// entry so result aggregation reports exactly which moves landed.
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
            &entry.destination_repo.name,
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

impl Git
{
    fn mv(
        &self,
        repo_path: &Path,
        source: &Path,
        destination: &Path
    ) -> Result<()>
    {
        let output =
            self.path_output(repo_path, ["mv", "--"], [source, destination])?;

        if !output.status.success()
        {
            bail!(
                "git mv failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }

    fn mv_to_directory(
        &self,
        repo_path: &Path,
        sources: &[PathBuf],
        destination: &Path
    ) -> Result<()>
    {
        let paths = sources
            .iter()
            .map(PathBuf::as_path)
            .chain(std::iter::once(destination));
        let output = self.path_output(repo_path, ["mv", "--"], paths)?;

        if !output.status.success()
        {
            bail!(
                "git mv failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }

    fn ensure_tracked(&self, repo_path: &Path, path: &Path) -> Result<()>
    {
        let output = self.path_output(
            repo_path,
            ["ls-files", "--error-unmatch", "--"],
            [path]
        )?;

        if !output.status.success()
        {
            bail!(
                "source path is not tracked in '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};
    use crate::git::{RepoOutcome, ScriptedFake};
    use crate::test_support::vmr_fixture;
    use clap::error::ErrorKind;
    use std::sync::Arc;

    #[test]
    fn parses_mv_two_or_more_paths()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "mv", "one", "two", "three"]).unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Mv(MvArgs {
                sources,
                destination
            })) if sources == [PathBuf::from("one"), PathBuf::from("two")]
                && destination == *"three"
        ));
    }

    #[test]
    fn rejects_mv_without_two_paths_or_with_unsupported_options()
    {
        for args in [
            vec!["git-vmr", "mv"],
            vec!["git-vmr", "mv", "one"],
            vec!["git-vmr", "mv", "-r", "one", "two"],
            vec!["git-vmr", "mv", "--dry-run", "one", "two"],
            vec!["git-vmr", "mv", "-k", "one", "two"],
            vec!["git-vmr", "mv", "--force", "one", "two"]
        ]
        {
            // Act
            let err = Cli::parse_from(args).err().unwrap();

            // Assert
            assert!(
                matches!(
                    err.kind(),
                    ErrorKind::MissingRequiredArgument
                        | ErrorKind::UnknownArgument
                        | ErrorKind::InvalidValue
                        | ErrorKind::TooFewValues
                ),
                "unexpected error kind: {:?}",
                err.kind()
            );
        }
    }

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
    fn single_source_same_repo_move_uses_git_mv_directly()
    {
        // Arrange: no tracking probe is scripted, so the plan path would fail
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(
            ["mv", "--", "src/old.rs", "src/new.rs"],
            0,
            "",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let outcomes = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![PathBuf::from("backend/src/old.rs")],
            destination: PathBuf::from("backend/src/new.rs")
        })
        .unwrap();

        // Assert
        assert!(outcomes.outcomes.iter().all(|outcome| {
            matches!(outcome, Ok(RepoOutcome::Success(None)))
        }));
        let calls = fake.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].path, tmp.path().join("backend"));
        assert_eq!(
            calls[0].args,
            ["mv", "--", "src/old.rs", "src/new.rs"]
                .map(std::ffi::OsString::from)
        );
    }

    #[test]
    fn multi_source_same_repo_move_uses_one_batched_git_mv()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = ScriptedFake::new().on(
            ["mv", "--", "a.txt", "b.txt", "docs"],
            0,
            "",
            ""
        );
        let fake = tracked_file(&tmp, fake, "backend", "a.txt");
        let fake = Arc::new(tracked_file(&tmp, fake, "backend", "b.txt"));
        fs::create_dir(tmp.path().join("backend/docs")).unwrap();
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let outcomes = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![
                PathBuf::from("backend/a.txt"),
                PathBuf::from("backend/b.txt"),
            ],
            destination: PathBuf::from("backend/docs")
        })
        .unwrap();

        // Assert
        assert!(outcomes.outcomes.iter().all(|outcome| {
            matches!(outcome, Ok(RepoOutcome::Success(None)))
        }));
        let mv_calls = fake
            .calls()
            .into_iter()
            .filter(|invocation| invocation.args.first() == Some(&"mv".into()))
            .collect::<Vec<_>>();
        assert_eq!(mv_calls.len(), 1);
        assert_eq!(mv_calls[0].path, tmp.path().join("backend"));
        assert_eq!(
            mv_calls[0].args,
            ["mv", "--", "a.txt", "b.txt", "docs"]
                .map(std::ffi::OsString::from)
        );
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
        let outcomes = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![PathBuf::from("backend/config.toml")],
            destination: PathBuf::from("frontend/settings.toml")
        })
        .unwrap();

        // Assert: the file moved on disk and both repos staged their side
        assert!(outcomes.outcomes.iter().all(|outcome| {
            matches!(outcome, Ok(RepoOutcome::Success(None)))
        }));
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
        let result = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![
                PathBuf::from("backend/notes.md"),
                PathBuf::from("frontend/notes.md"),
            ],
            destination: PathBuf::from("backend/docs")
        });

        // Assert: planning fails and no source has moved
        let error = result.err().unwrap().to_string();
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
        let result = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![
                PathBuf::from("backend/a.txt"),
                PathBuf::from("backend/b.txt"),
            ],
            destination: PathBuf::from("frontend/missing")
        });

        // Assert
        assert!(
            result
                .err()
                .unwrap()
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
            .on(["add", "--", "docs/b.txt"], 0, "", "");
        let fake = tracked_file(&tmp, fake, "backend", "a.txt");
        let fake = Arc::new(tracked_file(&tmp, fake, "backend", "b.txt"));
        fs::create_dir(tmp.path().join("frontend/docs")).unwrap();
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let outcomes = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![
                PathBuf::from("backend/a.txt"),
                PathBuf::from("backend/b.txt"),
            ],
            destination: PathBuf::from("frontend/docs")
        })
        .unwrap();

        // Assert: the failure names its entry, the half-done state stays on
        // disk, and the other entry landed
        let failures = outcomes
            .outcomes
            .iter()
            .filter_map(|outcome| match outcome
            {
                Ok(RepoOutcome::Failure(message)) => Some(message),
                _ => None
            })
            .collect::<Vec<_>>();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].repo, "backend");
        assert!(
            failures[0].message.contains("failed to stage source deletion")
        );
        assert!(tmp.path().join("frontend/docs/a.txt").exists());
        assert!(tmp.path().join("frontend/docs/b.txt").exists());
    }

    #[test]
    fn failed_destination_staging_reports_the_destination_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake =
            ScriptedFake::new().on(["add", "--", "config.toml"], 0, "", "").on(
                ["add", "--", "settings.toml"],
                1,
                "",
                "fatal: unable to stage"
            );
        let fake = Arc::new(tracked_file(&tmp, fake, "backend", "config.toml"));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let outcomes = mv(&workspace, tmp.path(), &MvArgs {
            sources: vec![PathBuf::from("backend/config.toml")],
            destination: PathBuf::from("frontend/settings.toml")
        })
        .unwrap();

        // Assert
        let failure = outcomes
            .outcomes
            .iter()
            .find_map(|outcome| match outcome
            {
                Ok(RepoOutcome::Failure(message)) => Some(message),
                _ => None
            })
            .unwrap();
        assert_eq!(failure.repo, "frontend");
        assert!(
            failure.message.contains("failed to stage destination addition")
        );
    }
}
