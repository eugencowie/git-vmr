pub mod mv;
pub mod worktree_root;

use crate::git::{Git, GitCommandResult};
use crate::render::{self, Rendered};
use crate::vmr::Vmr;
pub use crate::vmr::{Repo, resolve_target};
use anyhow::Result;
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

    /// Runs a git operation in every child repo in parallel, then renders
    /// the aggregated outcomes.
    pub fn run<F>(&self, op: F) -> Result<Rendered>
    where F: Fn(&Git, &Repo) -> GitCommandResult + Sync
    {
        self.run_in(&self.repos, op)
    }

    /// Runs a git operation in an explicit subset of child repos in
    /// parallel, then renders the aggregated outcomes.
    pub fn run_in<F>(&self, repos: &[Repo], op: F) -> Result<Rendered>
    where F: Fn(&Git, &Repo) -> GitCommandResult + Sync
    {
        render::outcomes(
            repos.par_iter().map(|repo| op(self.git, repo)).collect::<Vec<_>>()
        )
    }

    /// Routes the scope to its owning child repos, runs a git operation in
    /// each in parallel, then renders the aggregated outcomes.
    pub fn run_routed<F>(
        &self,
        working_dir: &Path,
        scope: Scope,
        op: F
    ) -> Result<Rendered>
    where
        F: Fn(&Git, &Repo, &[PathBuf]) -> GitCommandResult + Sync
    {
        let routed =
            self.route(working_dir, scope)?.into_iter().collect::<Vec<_>>();

        render::outcomes(
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
        resolve_target(working_dir, path) == self.vmr.path
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;
    use crate::test_support::vmr_fixture;
    use std::sync::Arc;

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

        // Assert: the shared failure groups to one line, and the repo list
        // is omitted because it covers every repo in scope.
        assert_eq!(err.to_string(), "merge failed");
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
}
