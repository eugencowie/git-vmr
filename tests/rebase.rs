mod common;

use common::{
    git, git_command, git_output, git_vmr, init_repo, init_vmr, write_commit
};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn setup_repo_with_initial_commit(path: &Path)
{
    init_repo(path);
    write_commit(path, "README.md", "initial\n", "initial");
}

fn create_rebasable_history(path: &Path, upstream: &str)
{
    git(path, ["checkout", "-b", upstream]);
    write_commit(path, "upstream.txt", "upstream\n", "upstream commit");
    git(path, ["checkout", "master"]);
    write_commit(path, "local.txt", "local\n", "local commit");
}

fn create_conflicting_rebase_history(path: &Path, upstream: &str)
{
    git(path, ["checkout", "-b", upstream]);
    write_commit(path, "conflict.txt", "upstream\n", "upstream conflict");
    git(path, ["checkout", "master"]);
    write_commit(path, "conflict.txt", "local\n", "local conflict");
}

fn has_rebase_state(path: &Path) -> bool
{
    path.join(".git/rebase-merge").exists()
        || path.join(".git/rebase-apply").exists()
}

fn head_contains(path: &Path, rev: &str) -> bool
{
    git_command()
        .args(["merge-base", "--is-ancestor", rev, "HEAD"])
        .current_dir(path)
        .status()
        .expect("failed to run git")
        .success()
}

fn head(path: &Path) -> String
{
    git_output(path, ["rev-parse", "HEAD"])
}

#[test]
fn rebase_clean_histories_across_multiple_child_repositories_succeeds_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_rebasable_history(&backend, "origin/main");
    create_rebasable_history(&frontend, "origin/main");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(head_contains(&backend, "origin/main"));
    assert!(head_contains(&frontend, "origin/main"));
}

#[test]
fn rebase_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend = tmp.path().join("backend");
    setup_repo_with_initial_commit(&backend);
    create_rebasable_history(&backend, "origin/main");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(head_contains(&backend, "origin/main"));
}

#[test]
fn rebase_missing_upstream_fails_only_affected_repository_and_still_attempts_others()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    setup_repo_with_initial_commit(&tools);
    create_rebasable_history(&backend, "origin/main");
    create_rebasable_history(&tools, "origin/main");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(frontend)"));

    assert!(head_contains(&backend, "origin/main"));
    assert!(head_contains(&tools, "origin/main"));
}

#[test]
fn rebase_conflict_preserves_rebase_state_and_does_not_stop_clean_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_conflicting_rebase_history(&backend, "origin/main");
    create_rebasable_history(&frontend, "origin/main");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));

    assert!(has_rebase_state(&backend));
    assert!(head_contains(&frontend, "origin/main"));
}

#[test]
fn rebase_successes_are_not_rolled_back_after_another_repository_fails()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_rebasable_history(&backend, "origin/main");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(frontend)"));

    assert!(head_contains(&backend, "origin/main"));
}

#[test]
fn rebase_reports_repository_suffixes_and_orders_failures_by_repository_name()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let beta = tmp.path().join("beta");
    let zeta = tmp.path().join("zeta");
    setup_repo_with_initial_commit(&alpha);
    setup_repo_with_initial_commit(&beta);
    setup_repo_with_initial_commit(&zeta);
    create_rebasable_history(&beta, "origin/main");

    git_vmr()
        .current_dir(tmp.path())
        .args(["rebase", "origin/main"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: invalid upstream 'origin/main' (alpha, zeta)\n"
        ));

    assert!(head_contains(&beta, "origin/main"));
}

#[test]
fn rebase_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_rebasable_history(&backend, "origin/main");

    git_vmr()
        .current_dir(&frontend)
        .args(["rebase", "origin/main"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(frontend)"));
    assert!(head_contains(&backend, "origin/main"));

    create_rebasable_history(&frontend, "origin/main");
    let before = head(&frontend);

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["rebase", "origin/main"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    assert_ne!(head(&frontend), before);
    assert!(head_contains(&frontend, "origin/main"));
}
