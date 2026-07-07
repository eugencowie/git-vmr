use crate::git;
use crate::git::Git;
use crate::render::{self, Rendered, WorktreeRootEntry};
use crate::vmr::Vmr;
use crate::workspace::{Repo, Workspace, resolve_target};
use anyhow::{Context, Result};
use path_clean::PathClean;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn list(workspace: &Workspace) -> Result<Rendered>
{
    let repo_names = workspace
        .repos()
        .iter()
        .map(|repo| repo.name.clone())
        .collect::<Vec<_>>();

    let results = workspace.map(|git, repo| {
        git.worktree_list(&repo.name, &repo.path)
            .map(|entries| (repo.name.clone(), entries))
    })?;

    let mut groups: BTreeMap<PathBuf, Vec<WorktreeRootEntry>> = BTreeMap::new();
    for (repo_name, entries) in results
    {
        for entry in entries
        {
            if let Some(root) =
                worktree_root(workspace.root(), &repo_name, &entry.path)
            {
                groups.entry(root).or_default().push(WorktreeRootEntry {
                    repo: repo_name.clone(),
                    head: entry.head,
                    state: entry.state
                });
            }
        }
    }

    Ok(render::worktree_list(groups, &repo_names).into())
}

pub fn add(
    workspace: &Workspace,
    working_dir: &Path,
    path: &Path,
    branch: Option<&str>,
    commit_ish: Option<&str>
) -> Result<Rendered>
{
    let target = resolve_target(working_dir, path);
    let mode = match (branch, commit_ish)
    {
        (Some(branch), commit_ish) => WorktreeAddMode::NewBranch {
            branch: branch.to_owned(),
            commit_ish: commit_ish.map(str::to_owned)
        },
        (None, Some(commit_ish)) =>
            WorktreeAddMode::CommitIsh(commit_ish.to_owned()),
        (None, None) => WorktreeAddMode::InferredBranch(
            target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .context("fatal: worktree target path must have a basename")?
        )
    };

    Vmr::create_worktree_root(&target)?;

    workspace.run(|git, repo| {
        let (branch, commit_ish) = match &mode
        {
            WorktreeAddMode::NewBranch { branch, commit_ish } =>
                (Some(branch.as_str()), commit_ish.as_deref()),
            WorktreeAddMode::CommitIsh(commit_ish) =>
                (None, Some(commit_ish.as_str())),
            WorktreeAddMode::InferredBranch(branch)
                if git.branch_exists(&repo.path, branch)? =>
                (None, Some(branch.as_str())),
            WorktreeAddMode::InferredBranch(branch) =>
                (Some(branch.as_str()), None),
        };

        git.worktree_add(
            &repo.name,
            &repo.path,
            &target.join(&repo.name),
            branch,
            commit_ish
        )
    })
}

enum WorktreeAddMode
{
    NewBranch
    {
        branch: String,
        commit_ish: Option<String>
    },
    CommitIsh(String),
    InferredBranch(String)
}

fn worktree_root(
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

pub fn remove(
    workspace: &Workspace,
    working_dir: &Path,
    path: &Path,
    force: u8,
    delete: bool,
    force_delete: bool
) -> Result<Rendered>
{
    let git = workspace.git();
    let target = resolve_target(working_dir, path);

    let removals = workspace.map(|git, repo| {
        // Wrap per-repo errors so map attempts every repository.
        Ok(removal_outcome(git, repo, &target, force, delete, force_delete))
    })?;

    let mut all_child_removals_succeeded = true;
    let mut results = Vec::new();
    let mut deletion_targets = Vec::new();
    for removal in removals
    {
        match removal
        {
            Ok(RemovalOutcome { repo_name, repo_path, branch, result }) =>
            {
                if !matches!(result, Ok(git::RepoOutcome::Success(_)))
                {
                    all_child_removals_succeeded = false;
                }
                else if let Some(branch) = branch
                {
                    deletion_targets.push((repo_name, repo_path, branch));
                }

                results.push(result);
            }
            Err(error) =>
            {
                all_child_removals_succeeded = false;
                results.push(Err(error));
            }
        }
    }

    if delete || force_delete
    {
        let branch_deletions = deletion_targets
            .par_iter()
            .map(|(repo_name, repo_path, branch)| {
                git.delete_branch(repo_name, repo_path, branch, force_delete)
            })
            .collect::<Vec<_>>();
        results.extend(branch_deletions);
    }

    let rendered = render::outcomes(results)?;

    if all_child_removals_succeeded
        && let Err(error) = Vmr::remove_worktree_root(&target)
    {
        return Err(render::fail(rendered, format!("{error:#}")));
    }

    Ok(rendered)
}

struct RemovalOutcome
{
    repo_name: String,
    repo_path: PathBuf,
    branch: Option<String>,
    result: git::GitCommandResult
}

fn removal_outcome(
    git: &Git,
    repo: &Repo,
    target: &Path,
    force: u8,
    delete: bool,
    force_delete: bool
) -> Result<RemovalOutcome>
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

    Ok(RemovalOutcome {
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
        .and_then(|entry| match entry.state
        {
            git::ChildWorktreeState::Branch(branch) => Some(branch),
            git::ChildWorktreeState::Detached => None
        }))
}

pub fn move_worktree(
    workspace: &Workspace,
    working_dir: &Path,
    path: &Path,
    new_path: &Path,
    force: u8
) -> Result<Rendered>
{
    let source = resolve_target(working_dir, path);
    let destination = resolve_target(working_dir, new_path);

    Vmr::create_worktree_root(&destination)?;

    let rendered = workspace.run(|git, repo| {
        git.worktree_move(
            &repo.name,
            &repo.path,
            &source.join(&repo.name),
            &destination.join(&repo.name),
            force
        )
    })?;

    // Keep the per-repo successes if dissolving the source root fails
    match Vmr::remove_worktree_root(&source)
    {
        Ok(()) => Ok(rendered),
        Err(error) => Err(render::fail(rendered, format!("{error:#}")))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;
    use crate::render::Failed;
    use crate::test_support::vmr_fixture;
    use std::ffi::OsString;
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
        let result = remove(&workspace, tmp.path(), &root, 0, true, false);

        // Assert: the root is not dissolved and the failure is the error,
        // while the backend success still renders
        let failed = result.unwrap_err().downcast::<Failed>().unwrap();
        assert!(failed.message.contains("modified or untracked files"));
        assert!(failed.message.contains("frontend"));
        assert!(failed.rendered.stdout.contains("Deleted branch feature"));
        assert!(failed.rendered.stdout.contains("backend"));
        assert!(root.join(".gitvmr").exists());

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
        let result = remove(&workspace, tmp.path(), &root, 0, true, false);

        // Assert: within each repo the lookup precedes the removal, because
        // removal destroys the worktree the lookup reads
        assert!(result.is_ok());
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
        let result = remove(&workspace, tmp.path(), &root, 0, true, false);

        // Assert: no branch deletion is attempted and the root dissolves
        assert!(result.is_ok());
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
        let result = remove(&workspace, tmp.path(), &root, 0, false, true);

        // Assert: the scripted -D rule answered, once per repo
        assert!(result.is_ok());
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
    fn partial_move_failure_keeps_both_roots_and_renders_successes()
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
                    "",
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
        let result =
            move_worktree(&workspace, tmp.path(), &source, &destination, 0);

        // Assert: the source root is not dissolved and the destination root
        // stays materialized for the children that did move
        let failed = result.unwrap_err().downcast::<Failed>().unwrap();
        assert!(failed.message.contains("locked working tree"));
        assert!(failed.message.contains("frontend"));
        assert!(source.join(".gitvmr").exists());
        assert!(destination.join(".gitvmr").exists());
    }
}
