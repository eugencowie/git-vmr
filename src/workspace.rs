mod routing;

use crate::git::{Git, GitCommandResult};
use crate::render::{self, Rendered};
pub use crate::vmr::Repo;
use crate::vmr::Vmr;
use anyhow::Result;
use rayon::prelude::*;
use routing::Router;
pub use routing::{AggregatePolicy, Scope};
use std::path::{Path, PathBuf};

/// A VMR opened by one command: the working dir it was opened from and
/// the child repos discovered at that moment, held fixed for the duration
/// of the command, plus the means to run git across them. The disk can
/// change mid-command; the workspace cannot.
pub struct Workspace<'a>
{
    git: &'a Git,
    vmr: Vmr,
    repos: Vec<Repo>,
    working_dir: PathBuf
}

impl<'a> Workspace<'a>
{
    /// Opens the VMR that owns `working_dir`, snapshotting its child repos.
    pub fn find(git: &'a Git, working_dir: &Path) -> Result<Workspace<'a>>
    {
        let vmr = Vmr::find(working_dir)?;
        let repos = vmr.repos()?;
        Ok(Self { git, vmr, repos, working_dir: working_dir.to_path_buf() })
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
            repos.par_iter().map(|repo| op(self.git, repo)).collect::<Vec<_>>(),
            repos.iter().map(|repo| repo.name.as_str())
        )
    }

    /// Routes the scope to its owning child repos, runs a git operation in
    /// each in parallel, then renders the aggregated outcomes.
    pub fn run_routed<F>(&self, scope: Scope, op: F) -> Result<Rendered>
    where F: Fn(&Git, &Repo, &[PathBuf]) -> GitCommandResult + Sync
    {
        let routed =
            self.router().route(scope)?.into_iter().collect::<Vec<_>>();

        render::outcomes(
            routed
                .par_iter()
                .map(|(repo, repo_paths)| op(self.git, repo, repo_paths))
                .collect::<Vec<_>>(),
            routed.iter().map(|(repo, _)| repo.name.as_str())
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

    /// Routes the scope to its owning child repos and gathers a value from
    /// each in parallel, in repo order, without reporting — for commands
    /// that assemble their own output from per-repo results.
    pub fn map_routed<T, F>(
        &self,
        scope: Scope,
        op: F
    ) -> Result<Vec<(Repo, T)>>
    where
        F: Fn(&Git, &Repo, &[PathBuf]) -> T + Sync,
        T: Send
    {
        let routed =
            self.router().route(scope)?.into_iter().collect::<Vec<_>>();

        Ok(routed
            .into_par_iter()
            .map(|(repo, repo_paths)| {
                let value = op(self.git, &repo, &repo_paths);
                (repo, value)
            })
            .collect())
    }

    /// Routes a single operand to its owning child repo. Single-operand
    /// routing always denies the aggregate path: one operand cannot expand
    /// to many child repos.
    pub fn route_single(&self, path: &Path) -> Result<(Repo, PathBuf)>
    {
        self.router().route_single(path)
    }

    /// Resolves a user-supplied path against the working dir the workspace
    /// was opened from and normalizes it lexically — the only form in
    /// which user paths reach the filesystem or routing.
    pub fn target(&self, path: &Path) -> PathBuf
    {
        self.router().target(path)
    }

    fn router(&self) -> Router<'_>
    {
        Router::new(&self.vmr.path, &self.repos, &self.working_dir)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{
        GitOutput, ScriptedFake, failure_message, quiet_success, stderr
    };
    use crate::test_support::vmr_fixture;
    use std::sync::Arc;

    // The run helpers take any per-repo operation; these tests script a
    // stand-in git invocation rather than borrowing a command's operation.
    fn outcome(repo: &Repo, output: GitOutput) -> GitCommandResult
    {
        Ok(
            if output.status.success()
            {
                quiet_success()
            }
            else
            {
                failure_message(&repo.name, stderr(&output))
            }
        )
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
            .run(|git, repo| outcome(repo, git.output(&repo.path, ["fetch"])?));

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
            .run(|git, repo| {
                outcome(repo, git.output(&repo.path, ["merge", "topic"])?)
            })
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
                outcome(repo, git.output(&repo.path, ["commit", "-m", "msg"])?)
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
            .run_routed(Scope::EntireVmr, |git, repo, paths| {
                outcome(
                    repo,
                    git.path_output(&repo.path, ["add", "--all", "--"], paths)?
                )
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
        let scope = Scope::Paths {
            paths: vec![PathBuf::from("backend/src/main.rs")],
            aggregate: AggregatePolicy::Allow
        };

        // Act
        workspace
            .run_routed(scope, |git, repo, paths| {
                outcome(
                    repo,
                    git.path_output(&repo.path, ["add", "--"], paths)?
                )
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
}
