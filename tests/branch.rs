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
fn branch_lists_local_branches_across_child_repositories()
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
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
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
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
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
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let frontend = tmp.path().join("frontend");
    init_repo(&frontend);
    commit_file(&frontend, "README.md");
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
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
        .stdout(predicate::str::contains(format!(
            "* (HEAD detached at {hash}) (frontend)"
        )))
        .stderr(predicate::str::is_empty());
}

#[test]
fn branch_fails_on_corrupted_git_dir()
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
        .arg("branch")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to read branch information"));
}
