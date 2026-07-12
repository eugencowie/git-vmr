use crate::git::{Git, GitCommandResult, RepoMessage, RepoOutcome};
use crate::vmr::Vmr;
pub use crate::vmr::{Repo, resolve_path};
use anyhow::{Result, bail};
use path_clean::PathClean;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What a path-taking command operates over: explicit paths, or the entire
/// VMR via the aggregate path.
pub enum Scope
{
    Paths(Vec<PathBuf>),
    EntireVmr
}

/// A VMR opened by one command: the child repos discovered at that moment,
/// held fixed for the duration of the command, plus the means to run git
/// across them. The disk can change mid-command; the workspace cannot.
pub struct Workspace<'a>
{
    git: &'a Git,
    vmr: Vmr,
    repos: Vec<Repo>
}

impl<'a> Workspace<'a>
{
    /// Opens the VMR that owns `working_dir`, snapshotting its child repos.
    pub fn find(git: &'a Git, working_dir: &Path) -> Result<Workspace<'a>>
    {
        let vmr = Vmr::find(working_dir)?;
        let repos = vmr.repos()?;
        Ok(Self { git, vmr, repos })
    }

    /// The VMR root directory.
    pub fn root(&self) -> &Path
    {
        &self.vmr.path
    }

    /// The git module, for commands that sequence their own operations.
    pub fn git(&self) -> &Git
    {
        self.git
    }

    /// The snapshotted child repos, sorted by name.
    pub fn repos(&self) -> &[Repo]
    {
        &self.repos
    }

    /// Runs a git operation in every child repo in parallel, then reports
    /// the aggregated outcomes.
    pub fn run<F>(&self, op: F) -> Result<()>
    where F: Fn(&Git, &Repo) -> GitCommandResult + Sync
    {
        self.run_in(&self.repos, op)
    }

    /// Runs a git operation in an explicit subset of child repos in
    /// parallel, then reports the aggregated outcomes.
    pub fn run_in<F>(&self, repos: &[Repo], op: F) -> Result<()>
    where F: Fn(&Git, &Repo) -> GitCommandResult + Sync
    {
        report(
            repos.par_iter().map(|repo| op(self.git, repo)).collect::<Vec<_>>()
        )
    }

    /// Routes the scope to its owning child repos, runs a git operation in
    /// each in parallel, then reports the aggregated outcomes.
    pub fn run_routed<F>(
        &self,
        working_dir: &Path,
        scope: Scope,
        op: F
    ) -> Result<()>
    where
        F: Fn(&Git, &Repo, &[PathBuf]) -> GitCommandResult + Sync
    {
        let routed =
            self.route(working_dir, scope)?.into_iter().collect::<Vec<_>>();

        report(
            routed
                .par_iter()
                .map(|(repo, repo_paths)| op(self.git, repo, repo_paths))
                .collect::<Vec<_>>()
        )
    }

    /// Gathers a value from every child repo in parallel, in repo order,
    /// without reporting.
    pub fn map<T, F>(&self, op: F) -> Result<Vec<T>>
    where
        F: Fn(&Git, &Repo) -> Result<T> + Sync,
        T: Send
    {
        self.repos.par_iter().map(|repo| op(self.git, repo)).collect()
    }

    /// Routes a single operand to its owning child repo, rejecting the
    /// aggregate path.
    pub fn route_single(
        &self,
        working_dir: &Path,
        path: &Path
    ) -> Result<(Repo, PathBuf)>
    {
        self.vmr.route_single_path(&self.repos, working_dir, path)
    }

    /// Whether a user-supplied path resolves to the aggregate path.
    pub fn is_aggregate(&self, working_dir: &Path, path: &Path) -> bool
    {
        resolve_path(working_dir, path).clean() == self.vmr.path
    }

    fn route(
        &self,
        working_dir: &Path,
        scope: Scope
    ) -> Result<BTreeMap<Repo, Vec<PathBuf>>>
    {
        let paths = match scope
        {
            Scope::Paths(paths) => paths,
            // The aggregate path expands to every snapshotted child repo
            Scope::EntireVmr => vec![self.vmr.path.clone()]
        };

        self.vmr.route_paths(&self.repos, working_dir, &paths)
    }
}

/// Aggregates per-repo outcomes: identical messages are grouped, successes
/// are printed, and failures become the command's error.
pub(crate) fn report(results: Vec<GitCommandResult>) -> Result<()>
{
    let mut successes = Vec::new();
    let mut failures = Vec::new();
    let mut errors = Vec::new();

    for result in results
    {
        match result
        {
            Ok(RepoOutcome::Success(Some(message))) => successes.push(message),
            Ok(RepoOutcome::Success(None)) =>
            {}
            Ok(RepoOutcome::Failure(message)) => failures.push(message),
            Err(error) => errors.push(error)
        }
    }

    for message in
        grouped_messages(successes, SuccessRepositoryFormat::NamesUntilLimit)
    {
        println!("{message}");
    }

    let mut rendered_errors =
        grouped_messages(failures, SuccessRepositoryFormat::NamesWithCount);
    rendered_errors.extend(errors.into_iter().map(|error| error.to_string()));

    if !rendered_errors.is_empty()
    {
        bail!(rendered_errors.join("\n"));
    }

    Ok(())
}

const REPOSITORY_NAME_LIMIT: usize = 5;

enum SuccessRepositoryFormat
{
    NamesUntilLimit,
    NamesWithCount
}

fn grouped_messages(
    messages: Vec<RepoMessage>,
    format: SuccessRepositoryFormat
) -> Vec<String>
{
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();

    for repo_message in messages
    {
        if let Some((_, repos)) = groups
            .iter_mut()
            .find(|(message, _)| message == &repo_message.message)
        {
            repos.push(repo_message.repo);
        }
        else
        {
            groups.push((repo_message.message, vec![repo_message.repo]));
        }
    }

    groups
        .into_iter()
        .map(|(message, repos)| {
            format!("{message} {}", repository_suffix(&repos, &format))
        })
        .collect()
}

fn repository_suffix(
    repos: &[String],
    format: &SuccessRepositoryFormat
) -> String
{
    if repos.len() == 1
    {
        return format!("({})", repos[0]);
    }

    match format
    {
        SuccessRepositoryFormat::NamesUntilLimit
            if repos.len() > REPOSITORY_NAME_LIMIT =>
        {
            format!("({} repos)", repos.len())
        }
        SuccessRepositoryFormat::NamesUntilLimit =>
            format!("({})", repos.join(", ")),
        SuccessRepositoryFormat::NamesWithCount
            if repos.len() > REPOSITORY_NAME_LIMIT =>
        {
            format!(
                "({} repos: {}, ...)",
                repos.len(),
                repos[..REPOSITORY_NAME_LIMIT].join(", ")
            )
        }
        SuccessRepositoryFormat::NamesWithCount =>
            format!("({})", repos.join(", ")),
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;
    use anyhow::anyhow;
    use std::fs;
    use std::sync::Arc;

    fn vmr_fixture() -> tempfile::TempDir
    {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".gitvmr")).unwrap();
        fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
        fs::create_dir_all(tmp.path().join("frontend/.git")).unwrap();
        tmp
    }

    fn repo_message(repo: &str, message: &str) -> RepoMessage
    {
        RepoMessage { repo: repo.to_owned(), message: message.to_owned() }
    }

    #[test]
    fn find_snapshots_child_repos_sorted_by_name()
    {
        // Arrange
        let tmp = vmr_fixture();
        let git = Git::subprocess();

        // Act
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Assert
        assert_eq!(workspace.root(), tmp.path());
        assert_eq!(
            workspace
                .repos()
                .iter()
                .map(|repo| repo.name.as_str())
                .collect::<Vec<_>>(),
            vec!["backend", "frontend"]
        );
    }

    #[test]
    fn run_invokes_operation_in_every_child_repo()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(["fetch"], 0, "", ""));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let result = workspace
            .run(|git, repo| git.fetch(&repo.name, &repo.path, None, &[]));

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
    fn run_reports_grouped_failures_as_the_command_error()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake =
            ScriptedFake::new().on(["merge", "topic"], 1, "", "merge failed");
        let git = Git::with(fake);
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let err = workspace
            .run(|git, repo| git.merge(&repo.name, &repo.path, "topic"))
            .unwrap_err();

        // Assert
        assert_eq!(err.to_string(), "merge failed (backend, frontend)");
    }

    #[test]
    fn run_in_only_touches_the_given_subset()
    {
        // Arrange
        let tmp = vmr_fixture();
        let fake = Arc::new(ScriptedFake::new().on(
            ["commit", "-m", "msg"],
            0,
            "",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();
        let subset = vec![workspace.repos()[0].clone()];

        // Act
        workspace
            .run_in(&subset, |git, repo| {
                git.commit(&repo.name, &repo.path, "msg")
            })
            .unwrap();

        // Assert
        assert_eq!(
            fake.calls()
                .into_iter()
                .map(|invocation| invocation.path)
                .collect::<Vec<_>>(),
            vec![tmp.path().join("backend")]
        );
    }

    #[test]
    fn run_routed_expands_entire_vmr_scope_to_every_child_repo()
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
        workspace
            .run_routed(tmp.path(), Scope::EntireVmr, |git, repo, paths| {
                git.add(&repo.name, &repo.path, paths, true, false, None)
            })
            .unwrap();

        // Assert
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
    fn run_routed_only_touches_owning_child_repos()
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
        let scope = Scope::Paths(vec![PathBuf::from("backend/src/main.rs")]);

        // Act
        workspace
            .run_routed(tmp.path(), scope, |git, repo, paths| {
                git.add(&repo.name, &repo.path, paths, false, false, None)
            })
            .unwrap();

        // Assert
        assert_eq!(
            fake.calls()
                .into_iter()
                .map(|invocation| invocation.path)
                .collect::<Vec<_>>(),
            vec![tmp.path().join("backend")]
        );
    }

    #[test]
    fn map_gathers_values_in_repo_order()
    {
        // Arrange
        let tmp = vmr_fixture();
        let git = Git::subprocess();
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let names = workspace.map(|_git, repo| Ok(repo.name.clone())).unwrap();

        // Assert
        assert_eq!(names, vec!["backend", "frontend"]);
    }

    #[test]
    fn route_single_rejects_the_aggregate_path()
    {
        // Arrange
        let tmp = vmr_fixture();
        let git = Git::subprocess();
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let err =
            workspace.route_single(tmp.path(), Path::new(".")).unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("VMR root aggregate"));
    }

    #[test]
    fn is_aggregate_detects_the_vmr_root()
    {
        // Arrange
        let tmp = vmr_fixture();
        let git = Git::subprocess();
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Assert
        assert!(workspace.is_aggregate(tmp.path(), Path::new(".")));
        assert!(
            workspace
                .is_aggregate(&tmp.path().join("backend"), Path::new(".."))
        );
        assert!(!workspace.is_aggregate(tmp.path(), Path::new("backend")));
    }

    #[test]
    fn grouped_messages_combines_repositories_with_same_message()
    {
        // Arrange
        let messages = vec![
            repo_message("backend", "Already up to date."),
            repo_message("frontend", "Already up to date."),
            repo_message("tools", "Updating abc123..def456"),
        ];

        // Act
        let rendered = grouped_messages(
            messages,
            SuccessRepositoryFormat::NamesUntilLimit
        );

        // Assert
        assert_eq!(rendered, vec![
            "Already up to date. (backend, frontend)",
            "Updating abc123..def456 (tools)"
        ]);
    }

    #[test]
    fn grouped_success_messages_use_count_for_large_repository_sets()
    {
        // Arrange
        let messages = (1..=6)
            .map(|index| {
                repo_message(&format!("repo-{index}"), "Already up to date.")
            })
            .collect();

        // Act
        let rendered = grouped_messages(
            messages,
            SuccessRepositoryFormat::NamesUntilLimit
        );

        // Assert
        assert_eq!(rendered, vec!["Already up to date. (6 repos)"]);
    }

    #[test]
    fn grouped_failure_messages_keep_repository_sample_for_large_sets()
    {
        // Arrange
        let messages = (1..=6)
            .map(|index| {
                repo_message(&format!("repo-{index}"), "remote rejected")
            })
            .collect();

        // Act
        let rendered =
            grouped_messages(messages, SuccessRepositoryFormat::NamesWithCount);

        // Assert
        assert_eq!(rendered, vec![
            "remote rejected (6 repos: repo-1, repo-2, repo-3, repo-4, repo-5, ...)"
        ]);
    }

    #[test]
    fn report_succeeds_for_empty_or_quiet_success_results()
    {
        // Act
        let result = report(vec![Ok(RepoOutcome::Success(None))]);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn report_groups_failures_and_appends_errors()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Failure(repo_message(
                "backend",
                "error: branch not found"
            ))),
            Ok(RepoOutcome::Failure(repo_message(
                "frontend",
                "error: branch not found"
            ))),
            Err(anyhow!("fatal: transport failed")),
        ];

        // Act
        let err = report(results).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: branch not found (backend, frontend)\nfatal: transport failed"
        );
    }
}
