mod routing;

use crate::git::{Git, GitCommandResult, stderr};
use crate::render::{self, Rendered};
pub use crate::vmr::Repo;
use crate::vmr::{self, Vmr};
use anyhow::{Result, bail};
use rayon::prelude::*;
use routing::Router;
pub use routing::{AggregatePolicy, Scope};
use std::path::{Path, PathBuf};

/// A VMR opened by one command: the working dir it was opened from and
/// the child repos its manifest declares at that moment, held fixed for
/// the duration of the command, plus the means to run git across them. The
/// disk can change mid-command; the workspace cannot.
pub struct Workspace<'a>
{
    git: &'a Git,
    vmr: Vmr,
    /// The declared child repos that are cloned, sorted by path.
    repos: Vec<Repo>,
    /// The paths of declared child repos that are not cloned.
    missing: Vec<String>,
    warnings: Vec<String>,
    working_dir: PathBuf
}

impl<'a> Workspace<'a>
{
    /// Opens the VMR that owns `working_dir`, snapshotting the child repos
    /// its manifest declares and noting load warnings: declared repos that
    /// are not cloned, and declared repos the root repo does not ignore.
    pub fn open(git: &'a Git, working_dir: &Path) -> Result<Workspace<'a>>
    {
        let vmr = Vmr::find(working_dir)?;
        let manifest = vmr.manifest()?;

        let (repos, missing): (Vec<_>, Vec<_>) = manifest
            .declarations()
            .iter()
            .map(|declaration| Repo {
                name: declaration.path.clone(),
                path: vmr.path.join(&declaration.path)
            })
            .partition(|repo| repo.path.join(".git").exists());
        let missing =
            missing.into_iter().map(|repo| repo.name).collect::<Vec<_>>();

        let mut warnings = Vec::new();
        if !missing.is_empty()
        {
            warnings.push(format!(
                "{} declared in .gitvmr but not cloned; run 'git vmr clone' \
                 to clone them",
                quoted_list(&missing)
            ));
        }
        let declared = manifest
            .declarations()
            .iter()
            .map(|declaration| declaration.path.as_str())
            .collect::<Vec<_>>();
        match unignored(git, &vmr.path, &declared)
        {
            Ok(unignored) => warnings
                .extend(unignored.into_iter().map(|path| {
                format!(
                    "'{path}' is not ignored by the root repo; add '/{path}/' \
                     to .gitignore"
                )
            })),
            Err(error) => warnings.push(format!("{error:#}"))
        }

        Ok(Self {
            git,
            vmr,
            repos,
            missing,
            warnings,
            working_dir: working_dir.to_path_buf()
        })
    }

    /// Warnings raised while opening the workspace.
    pub fn warnings(&self) -> &[String]
    {
        &self.warnings
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
    ///
    /// A deliberate pass-through, like `target`: slices must not construct
    /// a `Router` or know it exists.
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
        Router::new(
            &self.vmr.path,
            &self.repos,
            &self.missing,
            &self.working_dir
        )
    }
}

/// The declared child repo paths the root repo does not ignore. A
/// `.gitignore` line of exactly `/<path>/` — the line git-vmr writes —
/// settles a path without spawning git; only the rest go through one
/// `git check-ignore`, so hand-written patterns still count.
fn unignored<'p>(
    git: &Git,
    root: &Path,
    paths: &[&'p str]
) -> Result<Vec<&'p str>>
{
    let gitignore = vmr::read_gitignore(root);
    let unlisted = paths
        .iter()
        .copied()
        .filter(|path| !vmr::has_ignore_line(&gitignore, path))
        .collect::<Vec<_>>();
    if unlisted.is_empty()
    {
        return Ok(unlisted);
    }

    // Trailing slashes mark each path as a directory for pattern matching
    let output = git.output(
        root,
        ["check-ignore", "--"]
            .into_iter()
            .map(str::to_owned)
            .chain(unlisted.iter().map(|path| format!("{path}/")))
    )?;
    if !matches!(output.status.code(), Some(0 | 1))
    {
        bail!(
            "failed to check which child repos are ignored: {}",
            stderr(&output)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ignored = stdout
        .lines()
        .map(|line| line.trim_end_matches('/'))
        .collect::<Vec<_>>();
    Ok(unlisted.into_iter().filter(|path| !ignored.contains(path)).collect())
}

/// Paths as a quoted, comma-separated list.
fn quoted_list(paths: &[String]) -> String
{
    paths.iter().map(|path| format!("'{path}'")).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{
        GitOutput, ScriptedFake, failure_message, quiet_success, stderr
    };
    use crate::test_support::{root_repo, vmr_fixture};
    use std::fs;
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
    fn open_labels_child_repos_by_manifest_path_sorted_by_path()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        root_repo(tmp.path(), &[], &["a", "b"]);
        fs::write(
            tmp.path().join(".gitvmr"),
            "[repo.zeta]\npath = \"a\"\nurl = \"u\"\n\n\
             [repo.alpha]\npath = \"b\"\nurl = \"u\"\n"
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("a/.git")).unwrap();
        fs::create_dir_all(tmp.path().join("b/.git")).unwrap();
        let git = Git::with(ScriptedFake::new());

        // Act
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Assert
        assert_eq!(workspace.repos(), [
            Repo { name: "a".to_owned(), path: tmp.path().join("a") },
            Repo { name: "b".to_owned(), path: tmp.path().join("b") }
        ]);
        assert!(workspace.warnings().is_empty());
    }

    #[test]
    fn open_leaves_out_undeclared_repos()
    {
        // Arrange
        let tmp = vmr_fixture();
        fs::create_dir_all(tmp.path().join("scratch/.git")).unwrap();
        let git = Git::with(ScriptedFake::new());

        // Act
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Assert
        assert_eq!(
            workspace
                .repos()
                .iter()
                .map(|repo| repo.name.as_str())
                .collect::<Vec<_>>(),
            ["backend", "frontend"]
        );
    }

    #[test]
    fn open_warns_once_about_declared_repos_that_are_not_cloned()
    {
        // Arrange: `docs` exists but is not a Git repository
        let tmp = tempfile::tempdir().unwrap();
        let declared = ["api", "backend", "docs"];
        root_repo(tmp.path(), &declared, &declared);
        fs::create_dir_all(tmp.path().join("backend/.git")).unwrap();
        fs::create_dir_all(tmp.path().join("docs")).unwrap();
        let git = Git::with(ScriptedFake::new());

        // Act
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Assert
        assert_eq!(
            workspace
                .repos()
                .iter()
                .map(|repo| repo.name.as_str())
                .collect::<Vec<_>>(),
            ["backend"]
        );
        assert_eq!(workspace.warnings(), [
            "'api', 'docs' declared in .gitvmr but not cloned; run 'git vmr \
             clone' to clone them"
        ]);
    }

    #[test]
    fn open_asks_git_only_about_children_without_an_ignore_line()
    {
        // Arrange: `frontend` is ignored by a hand-written pattern
        let tmp = vmr_fixture();
        fs::write(tmp.path().join(".gitignore"), "/backend/\n/*/\n").unwrap();
        let fake = Arc::new(ScriptedFake::new().on(
            ["check-ignore", "--", "frontend/"],
            0,
            "frontend/\n",
            ""
        ));
        let git = Git::with(Arc::clone(&fake));

        // Act
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Assert
        assert!(workspace.warnings().is_empty());
        assert_eq!(fake.calls().len(), 1);
        assert_eq!(fake.calls()[0].path, tmp.path());
    }

    #[test]
    fn open_warns_about_declared_repos_that_are_not_ignored()
    {
        // Arrange
        let tmp = vmr_fixture();
        fs::remove_file(tmp.path().join(".gitignore")).unwrap();
        let git = Git::with(ScriptedFake::new().on(
            ["check-ignore", "--", "backend/", "frontend/"],
            1,
            "",
            ""
        ));

        // Act
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Assert
        assert_eq!(workspace.warnings(), [
            "'backend' is not ignored by the root repo; add '/backend/' to \
             .gitignore",
            "'frontend' is not ignored by the root repo; add '/frontend/' to \
             .gitignore"
        ]);
    }

    #[test]
    fn open_fails_on_an_invalid_manifest()
    {
        // Arrange
        let tmp = vmr_fixture();
        fs::write(tmp.path().join(".gitvmr"), "[repo.x]\npath = \"x\"\n")
            .unwrap();
        let git = Git::with(ScriptedFake::new());

        // Act
        let Err(err) = Workspace::open(&git, tmp.path())
        else
        {
            panic!("expected an invalid manifest to fail opening");
        };

        // Assert
        let message = format!("{err:#}");
        assert!(message.starts_with("fatal: invalid manifest"), "{message}");
        assert!(message.contains("missing field `url`"), "{message}");
    }

    #[test]
    fn routing_into_a_repo_that_is_not_cloned_fails()
    {
        // Arrange
        let tmp = vmr_fixture();
        root_repo(tmp.path(), &["api", "backend", "frontend"], &[
            "api", "backend", "frontend"
        ]);
        let git = Git::with(ScriptedFake::new());
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Act
        let err = workspace.route_single(Path::new("api/src")).unwrap_err();

        // Assert
        assert!(format!("{err:#}").contains("'api' is not cloned"), "{err:#}");
    }

    #[test]
    fn open_snapshots_child_repos_at_the_vmr_root()
    {
        // Arrange
        let tmp = vmr_fixture();
        let git = Git::subprocess();

        // Act
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

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
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

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
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

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
        let workspace = Workspace::open(&git, tmp.path()).unwrap();
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
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

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
        let workspace = Workspace::open(&git, tmp.path()).unwrap();
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
        let workspace = Workspace::open(&git, tmp.path()).unwrap();

        // Act
        let names = workspace.map(|_git, repo| Ok(repo.name.clone())).unwrap();

        // Assert
        assert_eq!(names, vec!["backend", "frontend"]);
    }
}
