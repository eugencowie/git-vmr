use crate::git::{Git, GitCommandResult, Head, RepoOutcome};
use crate::vmr::Vmr;
use crate::workspace::{Repo, Workspace};
use anyhow::{Context, Result};
use path_clean::PathClean;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One child repo's entry under a worktree root, as gathered for listing.
#[derive(Clone, Eq, PartialEq)]
pub struct WorktreeRootEntry
{
    pub repo: String,
    pub head: Head
}

/// The result of a mutating root operation: the per-repo outcomes, plus the
/// fate of the root itself after them. Dissolving is per-root, not per-repo,
/// so it travels beside the repo outcomes rather than among them.
pub struct RootOutcomes
{
    pub outcomes: Vec<GitCommandResult>,
    /// `Err` only when dissolving the root was attempted and failed; a root
    /// deliberately kept (because a child operation failed) is `Ok`.
    pub dissolved: Result<()>
}

/// The worktree roots of one workspace. Owns the layout convention — each
/// child worktree lives at `<root>/<child repo name>` — and the
/// materialize → per-child → dissolve lifecycle.
pub struct WorktreeRoots<'a>
{
    workspace: &'a Workspace<'a>
}

impl<'a> Workspace<'a>
{
    /// The worktree roots of this workspace.
    pub fn worktree_roots(&'a self) -> WorktreeRoots<'a>
    {
        WorktreeRoots { workspace: self }
    }
}

impl<'a> WorktreeRoots<'a>
{
    /// Materializes the root at `target`, then adds one child worktree per
    /// child repo at `<target>/<repo name>`. With no explicit branch or
    /// commit-ish, the branch is inferred from the target's basename: each
    /// child repo checks the inferred branch out if it already has it, and
    /// creates it otherwise.
    pub fn add(
        &self,
        target: &Path,
        branch: Option<&str>,
        commit_ish: Option<&str>
    ) -> Result<Vec<GitCommandResult>>
    {
        let mode = match (branch, commit_ish)
        {
            (Some(branch), commit_ish) => AddMode::NewBranch {
                branch: branch.to_owned(),
                commit_ish: commit_ish.map(str::to_owned)
            },
            (None, Some(commit_ish)) =>
                AddMode::CommitIsh(commit_ish.to_owned()),
            (None, None) => AddMode::InferredBranch(
                target
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .context(
                        "fatal: worktree target path must have a basename"
                    )?
            )
        };

        Vmr::create_worktree_root(target)?;

        self.workspace.map(|git, repo| Ok(add_child(git, repo, target, &mode)))
    }

    /// Removes the child worktree at `<target>/<repo name>` from every child
    /// repo, then dissolves the root — but only when every child removal
    /// succeeded. With `delete` or `force_delete`, each child's branch is
    /// looked up before its worktree is removed (removal destroys what the
    /// lookup reads) and deleted afterwards, only where removal succeeded.
    pub fn remove(
        &self,
        target: &Path,
        force: u8,
        delete: bool,
        force_delete: bool
    ) -> Result<RootOutcomes>
    {
        let removals = self.workspace.map(|git, repo| {
            // Wrap per-repo errors so map attempts every repository.
            Ok(remove_child(git, repo, target, force, delete, force_delete))
        })?;

        let mut all_child_removals_succeeded = true;
        let mut outcomes = Vec::new();
        let mut deletion_targets = Vec::new();
        for removal in removals
        {
            match removal
            {
                Ok(ChildRemoval { repo_name, repo_path, branch, result }) =>
                {
                    if !matches!(result, Ok(RepoOutcome::Success(_)))
                    {
                        all_child_removals_succeeded = false;
                    }
                    else if let Some(branch) = branch
                    {
                        deletion_targets.push((repo_name, repo_path, branch));
                    }

                    outcomes.push(result);
                }
                Err(error) =>
                {
                    all_child_removals_succeeded = false;
                    outcomes.push(Err(error));
                }
            }
        }

        if delete || force_delete
        {
            let git = self.workspace.git();
            let branch_deletions = deletion_targets
                .par_iter()
                .map(|(repo_name, repo_path, branch)| {
                    git.delete_branch(
                        repo_name,
                        repo_path,
                        branch,
                        force_delete
                    )
                })
                .collect::<Vec<_>>();
            outcomes.extend(branch_deletions);
        }

        let dissolved = if all_child_removals_succeeded
        {
            Vmr::remove_worktree_root(target)
        }
        else
        {
            Ok(())
        };

        Ok(RootOutcomes { outcomes, dissolved })
    }

    /// Moves a worktree root: materializes the destination root, moves every
    /// child worktree from `<source>/<repo name>` to
    /// `<destination>/<repo name>`, then dissolves the source root. A source
    /// with failed child moves is deliberately kept, as is the destination
    /// for the children that did move.
    pub fn move_root(
        &self,
        source: &Path,
        destination: &Path,
        force: u8
    ) -> Result<RootOutcomes>
    {
        Vmr::create_worktree_root(destination)?;

        let outcomes = self.workspace.map(|git, repo| {
            Ok(git.worktree_move(
                &repo.name,
                &repo.path,
                &source.join(&repo.name),
                &destination.join(&repo.name),
                force
            ))
        })?;

        let all_child_moves_succeeded = outcomes
            .iter()
            .all(|outcome| matches!(outcome, Ok(RepoOutcome::Success(_))));

        let dissolved = if all_child_moves_succeeded
        {
            Vmr::remove_worktree_root(source)
        }
        else
        {
            Ok(())
        };

        Ok(RootOutcomes { outcomes, dissolved })
    }

    /// Gathers every child worktree across the workspace and groups it under
    /// the worktree root (or main VMR root) it belongs to.
    pub fn list(&self) -> Result<BTreeMap<PathBuf, Vec<WorktreeRootEntry>>>
    {
        let results = self.workspace.map(|git, repo| {
            git.worktree_list(&repo.name, &repo.path)
                .map(|entries| (repo.name.clone(), entries))
        })?;

        let mut groups: BTreeMap<PathBuf, Vec<WorktreeRootEntry>> =
            BTreeMap::new();
        for (repo_name, entries) in results
        {
            for entry in entries
            {
                if let Some(root) =
                    owning_root(self.workspace.root(), &repo_name, &entry.path)
                {
                    groups.entry(root).or_default().push(WorktreeRootEntry {
                        repo: repo_name.clone(),
                        head: entry.head
                    });
                }
            }
        }

        Ok(groups)
    }
}

struct ChildRemoval
{
    repo_name: String,
    repo_path: PathBuf,
    branch: Option<String>,
    result: GitCommandResult
}

fn remove_child(
    git: &Git,
    repo: &Repo,
    target: &Path,
    force: u8,
    delete: bool,
    force_delete: bool
) -> Result<ChildRemoval>
{
    let child_target = target.join(&repo.name).clean();
    let branch = if delete || force_delete
    {
        child_worktree_branch(git, &repo.name, &repo.path, &child_target)?
    }
    else
    {
        None
    };
    let result =
        git.worktree_remove(&repo.name, &repo.path, &child_target, force);

    Ok(ChildRemoval {
        repo_name: repo.name.clone(),
        repo_path: repo.path.clone(),
        branch,
        result
    })
}

fn child_worktree_branch(
    git: &Git,
    repo_name: &str,
    repo_path: &Path,
    child_target: &Path
) -> Result<Option<String>>
{
    let entries = git.worktree_list(repo_name, repo_path)?;
    let child_target = child_target.clean();

    Ok(entries
        .into_iter()
        .find(|entry| entry.path.clean() == child_target)
        .and_then(|entry| match entry.head
        {
            Head::Branch(branch) => Some(branch),
            // An unborn branch has no ref to delete, so it is skipped just
            // like a detached head.
            Head::Unborn(_) | Head::Detached(_) => None
        }))
}

enum AddMode
{
    NewBranch
    {
        branch: String,
        commit_ish: Option<String>
    },
    CommitIsh(String),
    InferredBranch(String)
}

fn add_child(
    git: &Git,
    repo: &Repo,
    target: &Path,
    mode: &AddMode
) -> GitCommandResult
{
    let (branch, commit_ish) = match mode
    {
        AddMode::NewBranch { branch, commit_ish } =>
            (Some(branch.as_str()), commit_ish.as_deref()),
        AddMode::CommitIsh(commit_ish) => (None, Some(commit_ish.as_str())),
        AddMode::InferredBranch(branch)
            if git.branch_exists(&repo.path, branch)? =>
            (None, Some(branch.as_str())),
        AddMode::InferredBranch(branch) => (Some(branch.as_str()), None)
    };

    git.worktree_add(
        &repo.name,
        &repo.path,
        &target.join(&repo.name),
        branch,
        commit_ish
    )
}

/// The reverse of the layout convention: the root a child worktree belongs
/// to. The main child sits under the VMR root itself; any other directory
/// named after the child repo whose parent bears a marker is that parent
/// root's child worktree.
fn owning_root(
    vmr_root: &Path,
    repo_name: &str,
    worktree_path: &Path
) -> Option<PathBuf>
{
    let main_child = vmr_root.join(repo_name);
    if worktree_path == main_child
    {
        return Some(vmr_root.to_owned());
    }

    if worktree_path.file_name()? != repo_name
    {
        return None;
    }

    let root = worktree_path.parent()?.to_owned();
    if Vmr::is_root(&root) { Some(root) } else { None }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::{Git, RepoOutcome, ScriptedFake};
    use crate::test_support::vmr_fixture;
    use crate::vmr::Vmr;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// A worktree root at `<vmr>/feature` plus the porcelain listing both
    /// child repos would answer for it.
    fn worktree_fixture(state: &str) -> (tempfile::TempDir, PathBuf, String)
    {
        let tmp = vmr_fixture();
        let root = tmp.path().join("feature");
        Vmr::create_worktree_root(&root).unwrap();

        let listing = format!(
            "worktree {backend}\0HEAD aaaa\0{state}\0\0worktree {frontend}\0HEAD bbbb\0{state}\0\0",
            backend = root.join("backend").display(),
            frontend = root.join("frontend").display()
        );

        (tmp, root, listing)
    }

    #[test]
    fn add_checks_out_the_inferred_branch_where_it_already_exists()
    {
        // Arrange: the inferred branch 'feature' exists in every child repo
        let tmp = vmr_fixture();
        let root = tmp.path().join("feature");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["show-ref", "--exists", "refs/heads/feature"], 0, "", "")
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("add"),
                        root.join("backend").into(),
                        OsString::from("feature")
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("add"),
                        root.join("frontend").into(),
                        OsString::from("feature")
                    ],
                    0,
                    "",
                    ""
                )
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let outcomes =
            workspace.worktree_roots().add(&root, None, None).unwrap();

        // Assert: the root is materialized, both children succeed, and no
        // repo creates the branch (no -b anywhere)
        assert!(root.join(".gitvmr").exists());
        assert_eq!(outcomes.len(), 2);
        assert!(
            outcomes
                .iter()
                .all(|outcome| matches!(outcome, Ok(RepoOutcome::Success(_))))
        );
        assert!(fake.calls().iter().all(|invocation| {
            !invocation.args.contains(&OsString::from("-b"))
        }));
    }

    #[test]
    fn add_creates_the_inferred_branch_where_it_is_missing()
    {
        // Arrange: no child repo has the inferred branch 'feature'
        let tmp = vmr_fixture();
        let root = tmp.path().join("feature");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["show-ref", "--exists", "refs/heads/feature"], 2, "", "")
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("add"),
                        OsString::from("-b"),
                        OsString::from("feature"),
                        root.join("backend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("add"),
                        OsString::from("-b"),
                        OsString::from("feature"),
                        root.join("frontend").into()
                    ],
                    0,
                    "",
                    ""
                )
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let outcomes =
            workspace.worktree_roots().add(&root, None, None).unwrap();

        // Assert
        assert_eq!(outcomes.len(), 2);
        assert!(
            outcomes
                .iter()
                .all(|outcome| matches!(outcome, Ok(RepoOutcome::Success(_))))
        );
    }

    fn calls_for(fake: &ScriptedFake, repo_path: &Path) -> Vec<Vec<OsString>>
    {
        fake.calls()
            .into_iter()
            .filter(|invocation| invocation.path == repo_path)
            .map(|invocation| invocation.args)
            .collect()
    }

    #[test]
    fn failed_child_removal_keeps_root_and_branch_and_reports_successes()
    {
        // Arrange: backend removal succeeds, frontend removal fails
        let (tmp, root, listing) =
            worktree_fixture("branch refs/heads/feature");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["worktree", "list", "--porcelain", "-z"], 0, &listing, "")
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("backend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("frontend").into()
                    ],
                    1,
                    "",
                    "fatal: 'frontend' contains modified or untracked files"
                )
                .on(["branch", "-d", "feature"], 0, "", "")
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let removal =
            workspace.worktree_roots().remove(&root, 0, true, false).unwrap();

        // Assert: the root is deliberately kept (not a dissolve failure),
        // and exactly one outcome is the frontend's failure
        assert!(removal.dissolved.is_ok());
        assert!(root.join(".gitvmr").exists());
        let failures = removal
            .outcomes
            .iter()
            .filter_map(|outcome| match outcome
            {
                Ok(RepoOutcome::Failure(message)) => Some(message),
                _ => None
            })
            .collect::<Vec<_>>();
        assert_eq!(failures.len(), 1);
        assert!(failures[0].message.contains("modified or untracked files"));

        // Only the repo whose removal succeeded deletes its branch
        let branch_calls = fake
            .calls()
            .into_iter()
            .filter(|invocation| {
                invocation.args.first() == Some(&OsString::from("branch"))
            })
            .collect::<Vec<_>>();
        assert_eq!(branch_calls.len(), 1);
        assert_eq!(branch_calls[0].path, tmp.path().join("backend"));
    }

    #[test]
    fn branch_lookup_happens_before_each_child_removal()
    {
        // Arrange: both removals succeed
        let (tmp, root, listing) =
            worktree_fixture("branch refs/heads/feature");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["worktree", "list", "--porcelain", "-z"], 0, &listing, "")
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("backend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("frontend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(["branch", "-d", "feature"], 0, "", "")
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let removal =
            workspace.worktree_roots().remove(&root, 0, true, false).unwrap();

        // Assert: within each repo the lookup precedes the removal, because
        // removal destroys the worktree the lookup reads
        assert!(removal.dissolved.is_ok());
        for repo in ["backend", "frontend"]
        {
            let calls = calls_for(&fake, &tmp.path().join(repo));
            assert_eq!(calls[0][..2], [
                OsString::from("worktree"),
                OsString::from("list")
            ]);
            assert_eq!(calls[1][..2], [
                OsString::from("worktree"),
                OsString::from("remove")
            ]);
        }

        // A fully-successful removal dissolves the worktree root
        assert!(!root.exists());
    }

    #[test]
    fn detached_child_worktrees_skip_branch_deletion()
    {
        // Arrange: both children are detached
        let (tmp, root, listing) = worktree_fixture("detached");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["worktree", "list", "--porcelain", "-z"], 0, &listing, "")
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("backend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("frontend").into()
                    ],
                    0,
                    "",
                    ""
                )
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let removal =
            workspace.worktree_roots().remove(&root, 0, true, false).unwrap();

        // Assert: no branch deletion is attempted and the root dissolves
        assert!(removal.dissolved.is_ok());
        assert!(fake.calls().iter().all(|invocation| {
            invocation.args.first() != Some(&OsString::from("branch"))
        }));
        assert!(!root.exists());
    }

    #[test]
    fn force_delete_passes_the_force_flag_to_branch_deletion()
    {
        // Arrange
        let (tmp, root, listing) =
            worktree_fixture("branch refs/heads/feature");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(["worktree", "list", "--porcelain", "-z"], 0, &listing, "")
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("backend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("frontend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(["branch", "-D", "feature"], 0, "", "")
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let removal =
            workspace.worktree_roots().remove(&root, 0, false, true).unwrap();

        // Assert: the scripted -D rule answered, once per repo
        assert!(removal.dissolved.is_ok());
        let deletions = fake
            .calls()
            .into_iter()
            .filter(|invocation| {
                invocation.args.first() == Some(&OsString::from("branch"))
            })
            .count();
        assert_eq!(deletions, 2);
    }

    #[test]
    fn dissolve_failure_reports_alongside_successful_child_removals()
    {
        // Arrange: both removals succeed, but the marker is an undeletable
        // non-empty directory, so dissolving the root fails
        let (tmp, root, _) = worktree_fixture("");
        std::fs::remove_file(root.join(".gitvmr")).unwrap();
        std::fs::create_dir(root.join(".gitvmr")).unwrap();
        std::fs::write(root.join(".gitvmr").join("config"), "").unwrap();
        let fake = Arc::new(
            ScriptedFake::new()
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("backend").into()
                    ],
                    0,
                    "",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("remove"),
                        root.join("frontend").into()
                    ],
                    0,
                    "",
                    ""
                )
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let removal =
            workspace.worktree_roots().remove(&root, 0, false, false).unwrap();

        // Assert: the child successes are kept beside the dissolve failure
        assert!(removal.dissolved.is_err());
        assert_eq!(removal.outcomes.len(), 2);
        assert!(
            removal
                .outcomes
                .iter()
                .all(|outcome| matches!(outcome, Ok(RepoOutcome::Success(_))))
        );
    }

    #[test]
    fn partial_move_failure_keeps_both_roots_and_reports_successes()
    {
        // Arrange: backend moves, frontend refuses
        let (tmp, source, _) = worktree_fixture("");
        let destination = tmp.path().join("renamed");
        let fake = Arc::new(
            ScriptedFake::new()
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("move"),
                        source.join("backend").into(),
                        destination.join("backend").into()
                    ],
                    0,
                    "Moved worktree",
                    ""
                )
                .on(
                    [
                        OsString::from("worktree"),
                        OsString::from("move"),
                        source.join("frontend").into(),
                        destination.join("frontend").into()
                    ],
                    1,
                    "",
                    "fatal: cannot move a locked working tree"
                )
        );
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let moved = workspace
            .worktree_roots()
            .move_root(&source, &destination, 0)
            .unwrap();

        // Assert: the source root is deliberately kept and the destination
        // root stays materialized for the children that did move
        assert!(moved.dissolved.is_ok());
        assert!(source.join(".gitvmr").exists());
        assert!(destination.join(".gitvmr").exists());
        let failures = moved
            .outcomes
            .iter()
            .filter_map(|outcome| match outcome
            {
                Ok(RepoOutcome::Failure(message)) => Some(message),
                _ => None
            })
            .collect::<Vec<_>>();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].repo, "frontend");
        assert!(failures[0].message.contains("locked working tree"));
        let successes = moved
            .outcomes
            .iter()
            .filter_map(|outcome| match outcome
            {
                Ok(RepoOutcome::Success(Some(message))) => Some(message),
                _ => None
            })
            .collect::<Vec<_>>();
        assert_eq!(successes.len(), 1);
        assert_eq!(successes[0].repo, "backend");
        assert_eq!(successes[0].message, "Moved worktree");
    }

    #[test]
    fn list_groups_child_worktrees_by_owning_root()
    {
        // Arrange
        let (tmp, root, listing) =
            worktree_fixture("branch refs/heads/feature");
        let fake = Arc::new(ScriptedFake::new().on(
            ["worktree", "list", "--porcelain", "-z"],
            0,
            &listing,
            ""
        ));
        let git = Git::with(Arc::clone(&fake));
        let workspace = Workspace::find(&git, tmp.path()).unwrap();

        // Act
        let groups = workspace.worktree_roots().list().unwrap();

        // Assert: both children group under the one root, in repo order
        assert_eq!(groups.keys().collect::<Vec<_>>(), vec![&root]);
        let entries = &groups[&root];
        assert_eq!(
            entries.iter().map(|entry| entry.repo.as_str()).collect::<Vec<_>>(),
            vec!["backend", "frontend"]
        );
        assert_eq!(entries[0].head, Head::Branch("feature".to_owned()));
    }
}
