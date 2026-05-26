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
}

fn commit_file(path: &Path, file: &str)
{
    fs::write(path.join(file), "content\n").expect("failed to write file");
    git(path, ["add", file]);
    git(path, ["commit", "-m", "initial"]);
}

#[test]
fn status_errors_outside_a_vmr()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: not a virtual monorepo (or any of the parent directories): .gitvmr"
            )
            .and(predicate::str::contains("fatal: fatal: not a virtual monorepo").not())
        );
}

#[test]
fn status_in_vmr_root_reports_child_repo_changes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");
    fs::write(repo.join("src.rs"), "changed\n").expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("On branch master")
                .and(predicate::str::contains("backend/src.rs"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_reports_clean_summary_for_clean_child_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout("On branch master\n\nnothing to commit, working tree clean\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_keeps_initial_repo_separate_from_committed_repo_on_same_branch()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let committed = tmp.path().join("committed");
    let new_repo = tmp.path().join("new-repo");
    init_repo(&committed);
    init_repo(&new_repo);
    commit_file(&committed, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "On branch master\n\nnothing to commit, working tree clean\n\n"
            )
            .and(predicate::str::contains(
                "On branch master (new-repo)\n\nNo commits yet\n"
            ))
            .and(
                predicate::str::contains(
                    "On branch master (committed, new-repo)"
                )
                .not()
            )
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_uses_nested_working_dir_for_discovery_and_relative_paths()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    fs::write(backend.join("src.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("../backend/src.rs"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_with_no_child_repos_succeeds_with_no_output()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non repo dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_reports_staged_changes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");
    fs::write(repo.join("new.rs"), "content\n").expect("failed to write file");
    git(&repo, ["add", "new.rs"]);

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Changes to be committed:")
                .and(predicate::str::contains("backend/new.rs"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_fails_on_corrupted_git_dir()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let repo = tmp.path().join("broken");
    fs::create_dir(&repo).expect("failed to create repo");
    fs::write(repo.join(".git"), "not a gitfile\n")
        .expect("failed to corrupt git dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to read git status"));
}
