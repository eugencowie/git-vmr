mod common;

use common::git_vmr;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn git(dir: &Path, args: &[&str])
{
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run git");
    assert!(
        output.status.success(),
        "git {:?} failed in '{}'\nstderr: {}\nstdout: {}",
        args,
        dir.display(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

fn git_output(dir: &Path, args: &[&str]) -> String
{
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run git");
    assert!(
        output.status.success(),
        "git {:?} failed in '{}'\nstderr: {}\nstdout: {}",
        args,
        dir.display(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );

    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn init_vmr(path: &Path)
{
    fs::create_dir(path.join(".gitvmr")).expect("failed to create marker");
}

fn init_bare_repo(path: &Path)
{
    fs::create_dir_all(path).expect("failed to create bare repo dir");
    git(path, &["init", "--bare"]);
}

fn clone_repo(source: &Path, destination: &Path)
{
    let parent = destination.parent().expect("clone destination has parent");
    git(parent, &[
        "clone",
        source.to_str().expect("source path should be UTF-8"),
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .expect("destination name should be UTF-8")
    ]);
    git(destination, &["config", "user.email", "test@example.com"]);
    git(destination, &["config", "user.name", "Test User"]);
}

fn write_commit(path: &Path, file: &str, content: &str, message: &str)
{
    fs::write(path.join(file), content).expect("failed to write file");
    git(path, &["add", file]);
    git(path, &["commit", "-m", message]);
}

fn setup_remote_and_clone(
    path: &Path,
    name: &str,
    file: &str
) -> (PathBuf, PathBuf)
{
    let remote = path.join("remotes").join(format!("{name}.git"));
    let work = path.join("vmr").join(name);
    init_bare_repo(&remote);
    clone_repo(&remote, &work);
    write_commit(&work, file, "initial\n", "initial");
    git(&work, &["push", "-u", "origin", "master"]);
    (remote, work)
}

fn head(path: &Path, rev: &str) -> String
{
    git_output(path, &["rev-parse", rev])
}

fn ref_exists(path: &Path, rev: &str) -> bool
{
    std::process::Command::new("git")
        .args(["rev-parse", "--verify", rev])
        .current_dir(path)
        .output()
        .expect("failed to run git")
        .status
        .success()
}

#[test]
fn push_publishes_commits_across_multiple_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (backend_remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "backend.txt");
    let (frontend_remote, frontend) =
        setup_remote_and_clone(tmp.path(), "frontend", "frontend.txt");
    write_commit(&backend, "backend.txt", "updated\n", "backend update");
    write_commit(&frontend, "frontend.txt", "updated\n", "frontend update");

    git_vmr()
        .current_dir(&vmr)
        .arg("push")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend_remote, "master"), head(&backend, "master"));
    assert_eq!(head(&frontend_remote, "master"), head(&frontend, "master"));
}

#[test]
fn push_forwards_repository_and_refspec_arguments()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "README.md");
    git(&backend, &["checkout", "-b", "release"]);
    write_commit(&backend, "release.txt", "release\n", "release");

    git_vmr()
        .current_dir(&vmr)
        .args(["push", "origin", "release:refs/heads/release"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&remote, "release"), head(&backend, "release"));
}

#[test]
fn push_treats_single_positional_argument_as_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "README.md");
    git(&backend, &["remote", "rename", "origin", "main"]);
    write_commit(&backend, "README.md", "updated\n", "update");

    git_vmr()
        .current_dir(&vmr)
        .args(["push", "main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&remote, "master"), head(&backend, "master"));
    assert!(!ref_exists(&backend, "origin/master"));
}

#[test]
fn push_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("push")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn push_failure_does_not_stop_successful_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (_backend_remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "README.md");
    let (frontend_remote, frontend) =
        setup_remote_and_clone(tmp.path(), "frontend", "README.md");
    git(&backend, &["remote", "remove", "origin"]);
    write_commit(&backend, "README.md", "backend\n", "backend update");
    write_commit(&frontend, "README.md", "frontend\n", "frontend update");

    git_vmr()
        .current_dir(&vmr)
        .args(["push", "origin", "master"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(frontend)"))
        .stderr(predicate::str::contains(
            "fatal: 'origin' does not appear to be a git repository (backend)"
        ));

    assert_eq!(head(&frontend_remote, "master"), head(&frontend, "master"));
}

#[test]
fn push_reports_success_failure_and_failure_order_with_repository_suffixes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (_alpha_remote, alpha) =
        setup_remote_and_clone(tmp.path(), "alpha", "README.md");
    let (backend_remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "README.md");
    let (_zeta_remote, zeta) =
        setup_remote_and_clone(tmp.path(), "zeta", "README.md");
    git(&alpha, &["remote", "remove", "origin"]);
    git(&zeta, &["remote", "remove", "origin"]);
    write_commit(&alpha, "README.md", "alpha\n", "alpha update");
    write_commit(&backend, "README.md", "backend\n", "backend update");
    write_commit(&zeta, "README.md", "zeta\n", "zeta update");

    git_vmr()
        .current_dir(&vmr)
        .args(["push", "origin", "master"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains(format!("To {}", backend_remote.display()))
                .and(predicate::str::contains("(backend)"))
        )
        .stderr(predicate::str::starts_with(
            "fatal: 'origin' does not appear to be a git repository (alpha, zeta)\n"
        ));
}

#[test]
fn push_delegates_missing_upstream_and_rejected_pushes_to_git_without_rollback()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (backend_remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "README.md");
    git(&backend, &["branch", "--unset-upstream"]);

    git_vmr()
        .current_dir(&vmr)
        .arg("push")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));

    let (frontend_remote, frontend) =
        setup_remote_and_clone(tmp.path(), "frontend", "README.md");
    let rejected_clone = tmp.path().join("rejected-clone");
    clone_repo(&backend_remote, &rejected_clone);
    write_commit(&rejected_clone, "README.md", "remote\n", "remote update");
    git(&rejected_clone, &["push", "origin", "master"]);
    git(&backend, &["branch", "--set-upstream-to=origin/master", "master"]);
    write_commit(&backend, "README.md", "local\n", "local update");
    write_commit(&frontend, "README.md", "frontend\n", "frontend update");

    git_vmr()
        .current_dir(&vmr)
        .arg("push")
        .assert()
        .failure()
        .stdout(predicate::str::contains("(frontend)"))
        .stderr(predicate::str::contains("(backend)"));

    assert_eq!(head(&frontend_remote, "master"), head(&frontend, "master"));
    assert_ne!(head(&backend, "master"), head(&backend, "origin/master"));
}

#[test]
fn push_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir_all(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let (backend_remote, backend) =
        setup_remote_and_clone(tmp.path(), "backend", "README.md");
    let (frontend_remote, frontend) =
        setup_remote_and_clone(tmp.path(), "frontend", "README.md");
    write_commit(&backend, "README.md", "backend once\n", "backend once");
    write_commit(&frontend, "README.md", "frontend once\n", "frontend once");

    git_vmr()
        .current_dir(&frontend)
        .arg("push")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    write_commit(&backend, "README.md", "backend twice\n", "backend twice");
    write_commit(&frontend, "README.md", "frontend twice\n", "frontend twice");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("push")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend_remote, "master"), head(&backend, "master"));
    assert_eq!(head(&frontend_remote, "master"), head(&frontend, "master"));
}
