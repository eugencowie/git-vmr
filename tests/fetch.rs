use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn git_vmr() -> Command
{
    Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary")
}

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
}

fn setup_remote_repo(path: &Path, file: &str) -> std::path::PathBuf
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
fn fetch_updates_remote_tracking_refs_across_multiple_child_repositories()
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
        .arg("fetch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!(
                "From {}",
                backend_source.display()
            ))
            .and(predicate::str::contains("(backend)"))
            .and(predicate::str::contains(format!(
                "From {}",
                frontend_source.display()
            )))
            .and(predicate::str::contains("(frontend)"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(
        head(&backend, "origin/master"),
        head(&backend_source, "master")
    );
    assert_eq!(
        head(&frontend, "origin/master"),
        head(&frontend_source, "master")
    );
}

#[test]
fn fetch_forwards_repository_and_refspec_arguments()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = setup_remote_repo(&tmp.path().join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend = vmr.join("backend");
    clone_repo(&source, &backend);

    git(&source, &["checkout", "-b", "release"]);
    write_commit(&source, "release.txt", "release\n", "release");

    git_vmr()
        .current_dir(&vmr)
        .args(["fetch", "origin", "release:refs/remotes/origin/release"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "origin/release"), head(&source, "release"));
}

#[test]
fn fetch_treats_single_positional_argument_as_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = setup_remote_repo(&tmp.path().join("source"), "README.md");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let backend = vmr.join("backend");
    clone_repo(&source, &backend);
    git(&backend, &["remote", "rename", "origin", "main"]);
    write_commit(&source, "README.md", "updated\n", "update");

    git_vmr()
        .current_dir(&vmr)
        .args(["fetch", "main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(backend)"))
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "main/master"), head(&source, "master"));
    assert!(!ref_exists(&backend, "origin/master"));
}

#[test]
fn fetch_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("fetch")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let source = setup_remote_repo(&tmp.path().join("source"), "README.md");
    let backend = tmp.path().join("backend");
    clone_repo(&source, &backend);

    git_vmr()
        .current_dir(tmp.path())
        .arg("fetch")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn fetch_failure_does_not_stop_successful_repositories()
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
        .args(["fetch", "origin"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("(frontend)"))
        .stderr(predicate::str::contains(
            "fatal: 'origin' does not appear to be a git repository (backend)"
        ));

    assert_eq!(
        head(&frontend, "origin/master"),
        head(&frontend_source, "master")
    );
}

#[test]
fn fetch_reports_success_failure_and_failure_order_with_repository_suffixes()
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
        .args(["fetch", "origin"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains(format!("From {}", source.display()))
                .and(predicate::str::contains("(backend)"))
        )
        .stderr(predicate::str::starts_with(
            "fatal: fatal: 'origin' does not appear to be a git repository (alpha)\n\
             fatal: fatal: 'origin' does not appear to be a git repository (zeta)\n"
        ));
}

#[test]
fn fetch_uses_nested_working_dir_and_global_c_option_for_discovery()
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
        .arg("fetch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("(backend)")
                .and(predicate::str::contains("(frontend)"))
        )
        .stderr(predicate::str::is_empty());

    write_commit(&source, "README.md", "updated twice\n", "update twice");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("fetch")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("(backend)")
                .and(predicate::str::contains("(frontend)"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend, "origin/master"), head(&source, "master"));
    assert_eq!(head(&frontend, "origin/master"), head(&source, "master"));
}
