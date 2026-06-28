mod common;

use common::git_vmr;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

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

fn run_git(command: &mut std::process::Command)
{
    let output = command.output().expect("failed to run git");
    assert!(
        output.status.success(),
        "git failed: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
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

fn setup_repo_behind_upstream(
    path: &Path,
    remote: &Path,
    seed: &Path,
    behind_count: usize
)
{
    fs::create_dir_all(remote.parent().unwrap())
        .expect("failed to create remote parent");
    run_git(
        std::process::Command::new("git").arg("init").arg("--bare").arg(remote)
    );
    run_git(
        std::process::Command::new("git").arg("clone").arg(remote).arg(seed)
    );
    git(seed, ["config", "user.email", "test@example.com"]);
    git(seed, ["config", "user.name", "Test User"]);
    write_commit(seed, "README.md", "initial\n", "initial");
    git(seed, ["branch", "-M", "develop"]);
    git(seed, ["push", "-u", "origin", "develop"]);
    git(remote, ["symbolic-ref", "HEAD", "refs/heads/develop"]);
    run_git(
        std::process::Command::new("git").arg("clone").arg(remote).arg(path)
    );
    git(path, ["switch", "-c", "other"]);

    for index in 1..=behind_count
    {
        write_commit(
            seed,
            "remote.txt",
            &format!("remote {index}\n"),
            &format!("remote {index}")
        );
    }

    git(seed, ["push"]);
    git(path, ["fetch", "origin"]);
}

fn count_occurrences(haystack: &str, needle: &str) -> usize
{
    haystack.match_indices(needle).count()
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
            predicate::str::contains("Switched to branch 'feature/auth'")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "feature/auth");
}

#[test]
fn switch_normalizes_behind_counts_before_grouping_success_output()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_behind_upstream(
        &backend,
        &tmp.path().join("remotes/backend.git"),
        &tmp.path().join("seeds/backend"),
        20
    );
    setup_repo_behind_upstream(
        &frontend,
        &tmp.path().join("remotes/frontend.git"),
        &tmp.path().join("seeds/frontend"),
        6
    );

    let assert = git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "develop"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert_eq!(
        count_occurrences(
            &stdout,
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        ),
        1
    );
    assert!(stdout.contains("(backend, frontend)"));
    assert!(!stdout.contains("by 20 commits"));
    assert!(!stdout.contains("by 6 commits"));
    assert_eq!(current_branch(&backend), "develop");
    assert_eq!(current_branch(&frontend), "develop");
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
            predicate::str::contains("Switched to branch 'feature/auth'")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("tools"))
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
            "fatal: invalid reference: feature/auth (alpha, zeta)\n"
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
            predicate::str::contains("Switched to branch 'feature/auth'")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
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
            predicate::str::contains("Switched to branch 'feature/auth'")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "feature/auth");
}

#[test]
fn switch_create_shared_branch_across_multiple_child_repositories_deduplicates_success()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_initial_commit(&backend);
    setup_repo_with_initial_commit(&frontend);

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "--create", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Switched to a new branch 'feature/auth'")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
    assert_eq!(current_branch(&frontend), "feature/auth");
}

#[test]
fn switch_create_short_form_creates_and_switches()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    setup_repo_with_initial_commit(&backend);

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "-c", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Switched to a new branch 'feature/auth'")
                .and(predicate::str::contains("(backend)"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
}

#[test]
fn switch_create_skips_non_git_children_and_empty_vmrs_succeed_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "--create", "feature/auth"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend = tmp.path().join("backend");
    setup_repo_with_initial_commit(&backend);

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "--create", "feature/auth"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Switched to a new branch 'feature/auth'")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&backend), "feature/auth");
}

#[test]
fn switch_create_existing_branch_fails_only_that_repository_and_still_attempts_others()

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

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "--create", "feature/auth"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Switched to a new branch 'feature/auth'")
                .and(predicate::str::contains("frontend"))
                .and(predicate::str::contains("tools"))
        )
        .stderr(predicate::str::contains(
            "fatal: a branch named 'feature/auth' already exists (backend)"
        ));

    assert_eq!(current_branch(&backend), "master");
    assert_eq!(current_branch(&frontend), "feature/auth");
    assert_eq!(current_branch(&tools), "feature/auth");
}

#[test]
fn switch_create_successes_are_not_rolled_back_after_failure()
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
        .args(["switch", "--create", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Switched to a new branch 'feature/auth'"
        ))
        .stderr(predicate::str::contains(
            "fatal: a branch named 'feature/auth' already exists (backend)"
        ));

    assert_eq!(current_branch(&backend), "master");
    assert_eq!(current_branch(&frontend), "feature/auth");
}

#[test]
fn switch_create_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let zeta = tmp.path().join("zeta");
    setup_repo_with_initial_commit(&alpha);
    setup_repo_with_initial_commit(&zeta);
    create_branch(&alpha, "feature/auth");
    create_branch(&zeta, "feature/auth");

    git_vmr()
        .current_dir(tmp.path())
        .args(["switch", "--create", "feature/auth"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: a branch named 'feature/auth' already exists (alpha, zeta)\n"
        ));

    assert_eq!(current_branch(&alpha), "master");
    assert_eq!(current_branch(&zeta), "master");
}
