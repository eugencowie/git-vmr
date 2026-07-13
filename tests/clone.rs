mod common;

use common::{commit_file, git_vmr, init_repo};
use predicates::prelude::*;
use std::fs;

#[test]
fn clone_uses_git_inferred_destination_from_non_vmr_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = tmp.path().join("project");
    let workspace = tmp.path().join("workspace");
    init_repo(&source);
    commit_file(&source, "README.md");
    fs::create_dir(&workspace).expect("failed to create workspace");

    git_vmr()
        .current_dir(&workspace)
        .arg("clone")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Cloning into 'project'"));

    assert!(workspace.join("project/.git").exists());
    assert!(workspace.join("project/README.md").is_file());
}

#[test]
fn clone_uses_explicit_destination_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = tmp.path().join("project");
    let workspace = tmp.path().join("workspace");
    init_repo(&source);
    commit_file(&source, "README.md");
    fs::create_dir(&workspace).expect("failed to create workspace");

    git_vmr()
        .current_dir(&workspace)
        .arg("clone")
        .arg(&source)
        .arg("copy")
        .assert()
        .success()
        .stderr(predicate::str::contains("Cloning into 'copy'"));

    assert!(workspace.join("copy/.git").exists());
    assert!(workspace.join("copy/README.md").is_file());
}

#[test]
fn clone_destination_is_relative_to_working_dir_argument()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = tmp.path().join("project");
    let workspace = tmp.path().join("workspace");
    let base = workspace.join("base");
    init_repo(&source);
    commit_file(&source, "README.md");
    fs::create_dir(&workspace).expect("failed to create workspace");
    fs::create_dir(&base).expect("failed to create base");

    git_vmr()
        .current_dir(&workspace)
        .arg("-C")
        .arg(&base)
        .arg("clone")
        .arg(&source)
        .arg("copy")
        .assert()
        .success()
        .stderr(predicate::str::contains("Cloning into 'copy'"));

    assert!(base.join("copy/.git").exists());
    assert!(base.join("copy/README.md").is_file());
    assert!(!workspace.join("copy").exists());
}

#[test]
fn clone_does_not_require_or_create_vmr_metadata()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = tmp.path().join("project");
    let workspace = tmp.path().join("workspace");
    init_repo(&source);
    commit_file(&source, "README.md");
    fs::create_dir(&workspace).expect("failed to create workspace");

    git_vmr()
        .current_dir(&workspace)
        .arg("clone")
        .arg(&source)
        .arg("copy")
        .assert()
        .success();

    assert!(workspace.join("copy/.git").exists());
    assert!(!workspace.join(".gitvmr").exists());
    assert!(!workspace.join("copy/.gitvmr").exists());
}

#[test]
fn clone_failed_git_process_exits_non_zero()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let workspace = tmp.path().join("workspace");
    fs::create_dir(&workspace).expect("failed to create workspace");

    git_vmr()
        .current_dir(&workspace)
        .arg("clone")
        .arg("foo")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr("fatal: repository 'foo' does not exist\n");

    assert!(!workspace.join("foo").exists());
}
