use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn git_vmr() -> Command
{
    Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary")
}

fn git<const N: usize>(dir: &Path, args: [&str; N])
{
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run git");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repo(path: &Path)
{
    fs::create_dir(path).expect("failed to create repo dir");
    git(path, ["init"]);
    git(path, ["config", "user.email", "test@example.com"]);
    git(path, ["config", "user.name", "Test User"]);
    fs::write(path.join("README.md"), "content\n")
        .expect("failed to write README");
    git(path, ["add", "README.md"]);
    git(path, ["commit", "-m", "initial"]);
}

#[test]
fn clone_uses_git_inferred_destination_from_non_vmr_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = tmp.path().join("project");
    let workspace = tmp.path().join("workspace");
    init_repo(&source);
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
    let missing = tmp.path().join("missing");
    fs::create_dir(&workspace).expect("failed to create workspace");

    git_vmr()
        .current_dir(&workspace)
        .arg("clone")
        .arg(&missing)
        .arg("copy")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal:")
                .and(predicate::str::contains("git clone"))
                .and(predicate::str::contains(missing.display().to_string()))
        );

    assert!(!workspace.join("copy").exists());
}
