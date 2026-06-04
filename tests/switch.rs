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
        "git failed: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

fn git_output<const N: usize>(dir: &Path, args: [&str; N]) -> String
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

    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn init_vmr(path: &Path)
{
    fs::create_dir(path.join(".gitvmr")).expect("failed to create marker");
}

fn init_repo(path: &Path)
{
    fs::create_dir(path).expect("failed to create repo dir");
    git(path, ["init"]);
    git(path, ["config", "user.email", "test@example.com"]);
    git(path, ["config", "user.name", "Test User"]);
}

fn write_commit(path: &Path, file: &str, content: &str, message: &str)
{
    fs::write(path.join(file), content).expect("failed to write file");
    git(path, ["add", file]);
    git(path, ["commit", "-m", message]);
}

fn setup_repo_with_initial_commit(path: &Path)
{
    init_repo(path);
    write_commit(path, "README.md", "initial\n", "initial");
}

fn create_branch(path: &Path, branch: &str)
{
    git(path, ["branch", branch]);
}

fn current_branch(path: &Path) -> String
{
    git_output(path, ["branch", "--show-current"])
}

#[test]
fn switch_shared_branch_across_multiple_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_branch(&backend, "feature/auth");
    create_branch(&frontend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Switched to branch 'feature/auth' (backend)"
            )
            .and(predicate::str::contains(
                "Switched to branch 'feature/auth' (frontend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "feature/auth");
}

#[test]
fn switch_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend = tmp.path().join("backend");
    setup_repo_with_initial_commit(&backend);
    create_branch(&backend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Switched to branch 'feature/auth' (backend)"
            )
            .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
}

#[test]
fn switch_missing_branch_fails_only_that_repository_and_still_attempts_others()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    setup_repo_with_initial_commit(&tools);
    create_branch(&backend, "feature/auth");
    create_branch(&tools, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains(
                "Switched to branch 'feature/auth' (backend)"
            )
            .and(predicate::str::contains(
                "Switched to branch 'feature/auth' (tools)"
            ))
        )
        .stderr(predicate::str::contains(
            "fatal: invalid reference: feature/auth (frontend)"
        ));

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "master");
    assert_eq!(current_branch(&tools), "feature/auth");
}

#[test]
fn switch_successes_are_not_rolled_back_after_later_failure()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_branch(&backend, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Switched to branch 'feature/auth' (backend)"
        ))
        .stderr(predicate::str::contains(
            "fatal: invalid reference: feature/auth (frontend)"
        ));

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "master");
}

#[test]
fn switch_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let zeta = tmp.path().join("zeta");
    setup_repo_with_initial_commit(&alpha);
    setup_repo_with_initial_commit(&zeta);

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: fatal: invalid reference: feature/auth (alpha)\n\
             fatal: fatal: invalid reference: feature/auth (zeta)\n"
        ));
}

#[test]
fn switch_dirty_worktree_protection_is_delegated_to_git()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    setup_repo_with_initial_commit(&backend);
    git(&backend, ["checkout", "-b", "feature/auth"]);
    write_commit(&backend, "README.md", "feature\n", "feature");
    git(&backend, ["checkout", "master"]);
    fs::write(backend.join("README.md"), "local\n")
        .expect("failed to write local change");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "error: Your local changes to the following files would be overwritten by checkout: (backend)"
        ));

    assert_eq!(current_branch(&backend), "master");
}

#[test]
fn switch_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);
    create_branch(&backend, "feature/auth");
    create_branch(&frontend, "feature/auth");

    git_vmr()
        .current_dir(&frontend)
        .args(["switch", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Switched to branch 'feature/auth' (backend)"
            )
            .and(predicate::str::contains(
                "Switched to branch 'feature/auth' (frontend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    git(&backend, ["switch", "master"]);
    git(&frontend, ["switch", "master"]);

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["switch", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Switched to branch 'feature/auth' (backend)"
            )
            .and(predicate::str::contains(
                "Switched to branch 'feature/auth' (frontend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "feature/auth");
}
