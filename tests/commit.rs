mod common;

use common::{commit_file, git, git_output, git_vmr, init_repo, init_vmr};
use predicates::prelude::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn stage_file(path: &Path, file: &str, content: &str)
{
    fs::write(path.join(file), content).expect("failed to write file");
    git(path, ["add", file]);
}

fn last_commit_subject(path: &Path) -> String
{
    git_output(path, ["log", "-1", "--pretty=%s"])
}

fn commit_count(path: &Path) -> String
{
    git_output(path, ["rev-list", "--count", "HEAD"])
}

#[test]
fn commit_staged_changes_in_multiple_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    stage_file(&backend, "backend.rs", "backend\n");
    stage_file(&frontend, "frontend.rs", "frontend\n");

    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "-m", "Implement new feature"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Implement new feature (backend)").and(
                predicate::str::contains("Implement new feature (frontend)")
            )
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(last_commit_subject(&backend), "Implement new feature");
    assert_eq!(last_commit_subject(&frontend), "Implement new feature");
}

#[test]
fn commit_skips_non_git_children_and_clean_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    stage_file(&backend, "backend.rs", "backend\n");

    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "-m", "Update backend"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Update backend")
                .and(predicate::str::contains("(backend)").not())
                .and(predicate::str::contains("frontend").not())
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(last_commit_subject(&backend), "Update backend");
    assert_eq!(commit_count(&frontend), "1");
}

#[test]
fn commit_accepts_short_and_long_message_options_and_requires_one()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    stage_file(&backend, "short.rs", "short\n");

    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "-m", "Short message"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Short message")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());
    assert_eq!(last_commit_subject(&backend), "Short message");

    stage_file(&backend, "long.rs", "long\n");
    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "--message", "Long message"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Long message")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());
    assert_eq!(last_commit_subject(&backend), "Long message");

    git_vmr()
        .current_dir(tmp.path())
        .arg("commit")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("required"));
}

#[test]
fn commit_preserves_initial_commit_summary_line()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    stage_file(&backend, "README.md", "initial\n");

    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "-m", "Initial import"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("(root-commit)")
                .and(predicate::str::contains("Initial import"))
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn commit_partial_failures_still_attempt_later_repositories()
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
    stage_file(&backend, "backend.rs", "backend\n");
    stage_file(&frontend, "frontend.rs", "frontend\n");
    stage_file(&tools, "tools.rs", "tools\n");
    install_failing_hook(&backend, "pre-commit hook declined");

    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "-m", "Implement new feature"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Implement new feature (frontend)")
                .and(predicate::str::contains("Implement new feature (tools)"))
        )
        .stderr(predicate::str::contains("pre-commit hook declined (backend)"));

    assert_eq!(last_commit_subject(&frontend), "Implement new feature");
    assert_eq!(last_commit_subject(&tools), "Implement new feature");
}

#[test]
fn commit_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let zeta = tmp.path().join("zeta");
    init_repo(&alpha);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&zeta, "README.md");
    stage_file(&alpha, "alpha.rs", "alpha\n");
    stage_file(&zeta, "zeta.rs", "zeta\n");
    install_failing_hook(&alpha, "alpha rejected");
    install_failing_hook(&zeta, "zeta rejected");

    git_vmr()
        .current_dir(tmp.path())
        .args(["commit", "-m", "Implement new feature"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "alpha rejected (alpha)\n\
             zeta rejected (zeta)\n"
        ));
}

#[test]
fn commit_reports_nothing_to_commit_when_no_child_repo_has_staged_changes()
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
        .args(["commit", "-m", "Implement new feature"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("error: nothing to commit").and(
            predicate::str::contains("fatal: error: nothing to commit").not()
        ));
}

#[test]
fn commit_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    stage_file(&backend, "backend.rs", "backend\n");

    git_vmr()
        .current_dir(&frontend)
        .args(["commit", "-m", "Nested commit"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Nested commit")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());
    assert_eq!(last_commit_subject(&backend), "Nested commit");

    stage_file(&backend, "backend2.rs", "backend2\n");
    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["commit", "-m", "C option commit"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("C option commit")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());
    assert_eq!(last_commit_subject(&backend), "C option commit");
}

fn install_failing_hook(repo: &Path, message: &str)
{
    let hook = repo.join(".git/hooks/pre-commit");
    fs::write(&hook, format!("#!/bin/sh\necho '{message}' >&2\nexit 1\n"))
        .expect("failed to write hook");

    #[cfg(unix)]
    {
        let mut permissions =
            fs::metadata(&hook).expect("failed to stat hook").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).expect("failed to chmod hook");
    }
}
