mod common;

use common::{git, git_output, git_vmr, init_repo, init_vmr, write_commit};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn create_mergeable_branch(path: &Path, branch: &str)
{
    git(path, ["checkout", "-b", branch]);
    write_commit(path, "feature.txt", "feature\n", "feature commit");
    git(path, ["checkout", "master"]);
}

fn create_conflicting_branch(path: &Path, branch: &str)
{
    git(path, ["checkout", "-b", branch]);
    write_commit(path, "conflict.txt", "feature\n", "feature conflict");
    git(path, ["checkout", "master"]);
    write_commit(path, "conflict.txt", "master\n", "master conflict");
}

fn setup_repo_with_initial_commit(path: &Path)
{
    init_repo(path);
    write_commit(path, "README.md", "initial\n", "initial");
}

fn branch_contains_head(path: &Path, branch: &str) -> bool
{
    std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", branch, "HEAD"])
        .current_dir(path)
        .status()
        .expect("failed to run git")
        .success()
}

fn has_merge_head(path: &Path) -> bool
{
    path.join(".git/MERGE_HEAD").exists()
}

fn commit_count(path: &Path) -> String
{
    git_output(path, ["rev-list", "--count", "HEAD"])
}

#[test]
fn merge_clean_branch_across_multiple_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_mergeable_branch(&backend, "feature/auth");
    create_mergeable_branch(&frontend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(backend, frontend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(branch_contains_head(&backend, "feature/auth"));
    assert!(branch_contains_head(&frontend, "feature/auth"));
}

#[test]
fn merge_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend = tmp.path().join("backend");
    setup_repo_with_initial_commit(&backend);
    create_mergeable_branch(&backend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(backend)").not())
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(branch_contains_head(&backend, "feature/auth"));
}

#[test]
fn merge_missing_ref_fails_only_that_repository_and_still_attempts_others()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    setup_repo_with_initial_commit(&tools);
    create_mergeable_branch(&backend, "feature/auth");
    create_mergeable_branch(&tools, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("tools"))
        )
        .stderr(predicate::str::contains(
            "merge: feature/auth - not something we can merge (frontend)"
        ));

    assert!(branch_contains_head(&backend, "feature/auth"));
    assert!(branch_contains_head(&tools, "feature/auth"));
    assert_eq!(commit_count(&frontend), "1");
}

#[test]
fn merge_conflict_leaves_repository_conflicted_and_does_not_stop_clean_repositories()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_conflicting_branch(&backend, "feature/auth");
    create_mergeable_branch(&frontend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(frontend)"))
        .stderr(
            predicate::str::contains("Auto-merging conflict.txt")
                .and(predicate::str::contains("(backend)"))
        );

    assert!(has_merge_head(&backend));
    assert!(branch_contains_head(&frontend, "feature/auth"));
}

#[test]
fn merge_successes_are_not_rolled_back_after_later_failure()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_mergeable_branch(&backend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::contains("(frontend)"));

    assert!(branch_contains_head(&backend, "feature/auth"));
}

#[test]
fn merge_reports_colored_repository_suffixes_and_orders_failures_by_repository_name()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let beta = tmp.path().join("beta");
    let zeta = tmp.path().join("zeta");
    setup_repo_with_initial_commit(&alpha);
    setup_repo_with_initial_commit(&beta);
    setup_repo_with_initial_commit(&zeta);
    create_mergeable_branch(&beta, "feature/auth");

    git_vmr()
        .env_remove("NO_COLOR")
        .env("CLICOLOR_FORCE", "1")
        .current_dir(tmp.path())
        .args(["merge", "feature/auth"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(beta)"))
        )
        .stderr(predicate::str::starts_with(
            "merge: feature/auth - not something we can merge \x1b[90m(alpha, zeta)\x1b[0m\n"
        ));
}

#[test]
fn merge_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_mergeable_branch(&backend, "feature/auth");

    git_vmr()
        .current_dir(&frontend)
        .args(["merge", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::contains("(frontend)"));
    assert!(branch_contains_head(&backend, "feature/auth"));

    create_mergeable_branch(&frontend, "feature/auth");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["merge", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Already up to date. (backend)")
                .and(predicate::str::contains("(frontend)"))
        )
        .stderr(predicate::str::is_empty());
    assert!(branch_contains_head(&frontend, "feature/auth"));
}
