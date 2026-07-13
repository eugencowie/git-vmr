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

fn init_repo(path: &Path)
{
    fs::create_dir(path).expect("failed to create repo dir");
    git(path, &["init"]);
    git(path, &["config", "user.email", "test@example.com"]);
    git(path, &["config", "user.name", "Test User"]);
}

fn write_commit(path: &Path, file: &str, content: &str, message: &str)
{
    fs::write(path.join(file), content).expect("failed to write file");
    git(path, &["add", file]);
    git(path, &["commit", "-m", message]);
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

fn setup_remote_repo(path: &Path, file: &str) -> PathBuf
{
    let source = path.to_owned();
    init_repo(&source);
    write_commit(&source, file, "initial\n", "initial");
    source
}

fn head(path: &Path, rev: &str) -> String
{
    git_output(path, &["rev-parse", rev])
}

fn file_content(path: &Path, file: &str) -> String
{
    fs::read_to_string(path.join(file)).expect("failed to read file")
}

#[test]
fn pull_updates_working_trees_across_multiple_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let sources = tmp.path().join("sources");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&sources).expect("failed to create sources dir");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);

    let backend_source =
        setup_remote_repo(&sources.join("backend"), "backend.txt");
    let frontend_source =
        setup_remote_repo(&sources.join("frontend"), "frontend.txt");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    clone_repo(&backend_source, &backend);
    clone_repo(&frontend_source, &frontend);
    write_commit(&backend_source, "backend.txt", "updated\n", "backend update");
    write_commit(
        &frontend_source,
        "frontend.txt",
        "updated\n",
        "frontend update"
    );

    git_vmr()
        .current_dir(&vmr)
        .arg("pull")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "HEAD"), head(&backend_source, "master"));
    assert_eq!(head(&frontend, "HEAD"), head(&frontend_source, "master"));
    assert_eq!(file_content(&backend, "backend.txt"), "updated\n");
    assert_eq!(file_content(&frontend, "frontend.txt"), "updated\n");
}

#[test]
fn pull_forwards_repository_and_refspec_arguments()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let sources = tmp.path().join("sources");
    fs::create_dir(&sources).expect("failed to create sources dir");
    let source = setup_remote_repo(&sources.join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend = vmr.join("backend");
    clone_repo(&source, &backend);

    git(&source, &["checkout", "-b", "release"]);
    write_commit(&source, "README.md", "release\n", "release");

    git_vmr()
        .current_dir(&vmr)
        .args(["pull", "origin", "release"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "HEAD"), head(&source, "release"));
    assert_eq!(file_content(&backend, "README.md"), "release\n");
}

#[test]
fn pull_treats_single_positional_argument_as_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let sources = tmp.path().join("sources");
    fs::create_dir(&sources).expect("failed to create sources dir");
    let source = setup_remote_repo(&sources.join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend = vmr.join("backend");
    clone_repo(&source, &backend);
    git(&backend, &["remote", "rename", "origin", "main"]);
    write_commit(&source, "README.md", "updated\n", "update");

    git_vmr()
        .current_dir(&vmr)
        .args(["pull", "main"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "HEAD"), head(&source, "master"));
}

#[test]
fn pull_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("pull")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let sources = tmp.path().join("sources");
    fs::create_dir(&sources).expect("failed to create sources dir");
    let source = setup_remote_repo(&sources.join("source"), "README.md");
    let backend = tmp.path().join("backend");
    clone_repo(&source, &backend);

    git_vmr()
        .current_dir(tmp.path())
        .arg("pull")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Already up to date.")
                .and(predicate::str::contains("(backend)").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn pull_failure_does_not_stop_successful_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let sources = tmp.path().join("sources");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&sources).expect("failed to create sources dir");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend_source =
        setup_remote_repo(&sources.join("backend"), "README.md");
    let frontend_source =
        setup_remote_repo(&sources.join("frontend"), "README.md");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    clone_repo(&backend_source, &backend);
    clone_repo(&frontend_source, &frontend);
    git(&backend, &["remote", "remove", "origin"]);
    write_commit(&frontend_source, "README.md", "updated\n", "update");

    git_vmr()
        .current_dir(&vmr)
        .args(["pull", "origin"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(frontend)"))
        .stderr(predicate::str::contains(
            "fatal: 'origin' does not appear to be a git repository (backend)"
        ));

    assert_eq!(head(&frontend, "HEAD"), head(&frontend_source, "master"));
}

#[test]
fn pull_reports_success_failure_and_failure_order_with_repository_suffixes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = setup_remote_repo(&tmp.path().join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let alpha = vmr.join("alpha");
    let backend = vmr.join("backend");
    let zeta = vmr.join("zeta");
    clone_repo(&source, &alpha);
    clone_repo(&source, &backend);
    clone_repo(&source, &zeta);
    git(&alpha, &["remote", "remove", "origin"]);
    git(&zeta, &["remote", "remove", "origin"]);
    write_commit(&source, "README.md", "updated\n", "update");

    git_vmr()
        .current_dir(&vmr)
        .args(["pull", "origin"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::starts_with(
            "fatal: 'origin' does not appear to be a git repository (alpha, zeta)\n"
        ));
}

#[test]
fn pull_delegates_missing_upstream_to_git()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = setup_remote_repo(&tmp.path().join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend = vmr.join("backend");
    clone_repo(&source, &backend);
    git(&backend, &["branch", "--unset-upstream"]);

    git_vmr()
        .current_dir(&vmr)
        .arg("pull")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "There is no tracking information for the current branch."
            )
            .and(predicate::str::contains("(backend)").not())
        );
}

#[test]
fn conflicted_pull_remains_for_user_resolution_without_rollback()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let sources = tmp.path().join("sources");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&sources).expect("failed to create sources dir");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend_source =
        setup_remote_repo(&sources.join("backend"), "README.md");
    let frontend_source =
        setup_remote_repo(&sources.join("frontend"), "README.md");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    clone_repo(&backend_source, &backend);
    clone_repo(&frontend_source, &frontend);
    git(&backend, &["config", "pull.rebase", "false"]);

    write_commit(&backend_source, "README.md", "remote\n", "remote update");
    write_commit(&backend, "README.md", "local\n", "local update");
    write_commit(&frontend_source, "README.md", "updated\n", "frontend update");

    git_vmr()
        .current_dir(&vmr)
        .arg("pull")
        .assert()
        .failure()
        .stdout(predicate::str::contains("(frontend)"))
        .stderr(predicate::str::contains("(backend)"));

    assert_eq!(head(&frontend, "HEAD"), head(&frontend_source, "master"));
    assert!(backend.join(".git/MERGE_HEAD").exists());
}

#[test]
fn pull_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = setup_remote_repo(&tmp.path().join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    clone_repo(&source, &backend);
    clone_repo(&source, &frontend);
    write_commit(&source, "README.md", "updated once\n", "update once");

    git_vmr()
        .current_dir(&frontend)
        .arg("pull")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(backend, frontend)").not())
        )
        .stderr(predicate::str::is_empty());

    write_commit(&source, "README.md", "updated twice\n", "update twice");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("pull")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Updating")
                .and(predicate::str::contains("(backend, frontend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "HEAD"), head(&source, "master"));
    assert_eq!(head(&frontend, "HEAD"), head(&source, "master"));
}
