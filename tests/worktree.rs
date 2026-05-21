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

fn current_branch(path: &Path) -> String
{
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .expect("failed to run git");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[test]
fn worktree_add_creates_child_worktrees_on_inferred_branch()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Preparing worktree (new branch 'wt') (backend, frontend)"
            )
            .or(predicate::str::contains(
                "Preparing worktree (new branch 'wt') (frontend, backend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&tmp.path().join("wt/backend")), "wt");
    assert_eq!(current_branch(&tmp.path().join("wt/frontend")), "wt");
}

#[test]
fn worktree_add_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    let backend = vmr.join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/docs").exists());
}

#[test]
fn worktree_add_explicit_checked_out_branch_fails_without_creating_inferred_branch()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "master"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: 'master' is already used by worktree"
            )
            .and(predicate::str::contains("(backend)"))
            .and(predicate::str::contains("(frontend)"))
        );

    assert!(!branch_exists(&backend, "wt"));
    assert!(!branch_exists(&frontend, "wt"));
}

#[test]
fn worktree_add_invalid_explicit_commit_ish_groups_failures()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "new"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "fatal: invalid reference: new (backend, frontend)"
        ));

    assert!(!branch_exists(&backend, "wt"));
    assert!(!branch_exists(&frontend, "wt"));
}

#[test]
fn worktree_add_best_effort_keeps_successful_child_worktrees()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    let tools = vmr.join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    git(&backend, ["branch", "wt"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("frontend")
                .and(predicate::str::contains("tools"))
                .and(predicate::str::contains("backend").not())
        )
        .stderr(predicate::str::contains(
            "fatal: a branch named 'wt' already exists (backend)"
        ));

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(tmp.path().join("wt/frontend").exists());
    assert!(tmp.path().join("wt/tools").exists());
}

#[test]
fn worktree_add_marker_enables_vmr_discovery_from_child_worktree()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();

    assert!(tmp.path().join("wt/.gitvmr").exists());
    git_vmr()
        .current_dir(tmp.path().join("wt/backend"))
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("On branch wt"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_add_uses_nested_working_dir_and_global_c_for_relative_target()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&frontend)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();

    assert!(vmr.join("wt/backend").exists());
    assert!(vmr.join("wt/frontend").exists());

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["worktree", "add", "../wt-c"])
        .assert()
        .success();

    assert!(vmr.join("wt-c/backend").exists());
    assert!(vmr.join("wt-c/frontend").exists());
}

#[test]
fn worktree_add_reports_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let alpha = vmr.join("alpha");
    let zeta = vmr.join("zeta");
    init_repo(&alpha);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&zeta, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "new"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: invalid reference: new (alpha, zeta)\n"
        ));
}
