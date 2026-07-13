mod common;

use common::{commit_file, git, git_vmr, init_repo, init_vmr};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn branch_exists(path: &Path, branch: &str) -> bool
{
    std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(path)
        .output()
        .expect("failed to run git")
        .status
        .success()
}

fn create_unmerged_branch(path: &Path, branch: &str)
{
    git(path, ["checkout", "-b", branch]);
    fs::write(path.join("feature.txt"), "feature\n")
        .expect("failed to write file");
    git(path, ["add", "feature.txt"]);
    git(path, ["commit", "-m", "feature"]);
    git(path, ["checkout", "master"]);
}

#[test]
fn branch_lists_local_branches_across_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    git(&backend, ["branch", "release/1.2"]);
    git(&frontend, ["branch", "release/1.2"]);
    git(&frontend, ["checkout", "-b", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .arg("branch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("* feature/auth (frontend)")
                .and(predicate::str::contains("* master"))
                .and(predicate::str::contains("  release/1.2"))
                .and(predicate::str::contains("origin").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn branch_skips_non_git_children_and_empty_vmr_outputs_nothing()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("branch")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .arg("branch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("* master")
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn branch_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    git(&backend, ["branch", "backend-only"]);

    git_vmr()
        .current_dir(&frontend)
        .arg("branch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("* master")
                .and(predicate::str::contains("  backend-only (backend)"))
        )
        .stderr(predicate::str::is_empty());

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("branch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("* master")
                .and(predicate::str::contains("  backend-only (backend)"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn branch_reports_detached_head_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let frontend = tmp.path().join("frontend");
    init_repo(&frontend);
    commit_file(&frontend, "README.md");
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .current_dir(&frontend)
        .output()
        .expect("failed to read head");
    assert!(output.status.success());
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    git(&frontend, ["checkout", "--detach"]);

    git_vmr()
        .current_dir(tmp.path())
        .arg("branch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("* (HEAD detached at {hash})"))
                .and(predicate::str::contains("(frontend)").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn branch_fails_on_corrupted_git_dir()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let repo = tmp.path().join("broken");
    fs::create_dir(&repo).expect("failed to create repo");
    fs::write(repo.join(".git"), "not a gitfile\n")
        .expect("failed to corrupt git dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("branch")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to read branch information"));
}

#[test]
fn branch_creates_branch_in_every_child_repository_with_no_output()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "feature/auth"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(branch_exists(&backend, "feature/auth"));
    assert!(branch_exists(&frontend, "feature/auth"));
}

#[test]
fn branch_create_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "feature/auth"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(branch_exists(&backend, "feature/auth"));
}

#[test]
fn branch_create_partial_failure_does_not_stop_other_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    git(&backend, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "fatal: a branch named 'feature/auth' already exists (backend)"
        ));

    assert!(branch_exists(&frontend, "feature/auth"));
    assert!(branch_exists(&tools, "feature/auth"));
}

#[test]
fn branch_create_omits_repository_suffix_when_every_repo_fails()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    git(&backend, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: a branch named 'feature/auth' already exists"
            )
            .and(predicate::str::contains("(backend)").not())
            .and(
                predicate::str::contains("fatal: failed to create branch")
                    .not()
            )
        );
}

#[test]
fn branch_create_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let clean = tmp.path().join("clean");
    let zeta = tmp.path().join("zeta");
    init_repo(&alpha);
    init_repo(&clean);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&clean, "README.md");
    commit_file(&zeta, "README.md");
    git(&alpha, ["branch", "feature/auth"]);
    git(&zeta, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: a branch named 'feature/auth' already exists (alpha, zeta)\n"
        ));
}

#[test]
fn branch_delete_safely_deletes_branch_in_every_child_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    git(&backend, ["branch", "feature/auth"]);
    git(&frontend, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-d", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deleted branch feature/auth")
                .and(predicate::str::contains("(backend, frontend)").not())
                .and(predicate::str::contains("(was").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(!branch_exists(&backend, "feature/auth"));
    assert!(!branch_exists(&frontend, "feature/auth"));
}

#[test]
fn branch_force_delete_deletes_branch_that_safe_delete_rejects()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    create_unmerged_branch(&backend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-d", "feature/auth"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: the branch 'feature/auth' is not fully merged"
        ));

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-D", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deleted branch feature/auth")
                .and(predicate::str::contains(" (backend)").not())
                .and(predicate::str::contains("(was").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(!branch_exists(&backend, "feature/auth"));
}

#[test]
fn branch_delete_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    git(&backend, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-d", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deleted branch feature/auth")
                .and(predicate::str::contains(" (backend)").not())
                .and(predicate::str::contains("(was").not())
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(!branch_exists(&backend, "feature/auth"));
}

#[test]
fn branch_delete_partial_failure_does_not_stop_other_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    create_unmerged_branch(&backend, "feature/auth");
    git(&frontend, ["branch", "feature/auth"]);
    git(&tools, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-d", "feature/auth"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Deleted branch feature/auth")
                .and(predicate::str::contains("frontend"))
                .and(predicate::str::contains("tools"))
                .and(predicate::str::contains("(was").not())
        )
        .stderr(predicate::str::contains(
            "error: the branch 'feature/auth' is not fully merged (backend)"
        ));

    assert!(branch_exists(&backend, "feature/auth"));
    assert!(!branch_exists(&frontend, "feature/auth"));
    assert!(!branch_exists(&tools, "feature/auth"));
}

#[test]
fn branch_delete_reports_missing_and_checked_out_branch_failures_concisely()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    git(&frontend, ["checkout", "-b", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-d", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "error: branch 'feature/auth' not found (backend)"
            )
            .and(predicate::str::contains(
                "error: cannot delete branch 'feature/auth' used by worktree at"
            ))
            .and(predicate::str::contains("(frontend)"))
        );
}

#[test]
fn branch_delete_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let has_branch = tmp.path().join("has-branch");
    let zeta = tmp.path().join("zeta");
    init_repo(&alpha);
    init_repo(&has_branch);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&has_branch, "README.md");
    commit_file(&zeta, "README.md");
    git(&has_branch, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["branch", "-d", "feature/auth"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with(
            "error: branch 'feature/auth' not found (alpha, zeta)\n"
        ));
}
