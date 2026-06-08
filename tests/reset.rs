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
        "git failed: {}\nstdout: {}",
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

fn setup_repo_with_two_commits(path: &Path)
{
    init_repo(path);
    write_commit(path, "README.md", "initial\n", "initial");
    write_commit(path, "README.md", "second\n", "second");
}

fn head(path: &Path) -> String
{
    git_output(path, ["rev-parse", "HEAD"])
}

fn rev(path: &Path, rev: &str) -> String
{
    git_output(path, ["rev-parse", rev])
}

fn status_short(path: &Path) -> String
{
    git_output(path, ["status", "--short"])
}

#[test]
fn reset_applies_across_multiple_child_repositories_and_skips_non_git_children()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_two_commits(&backend);
    setup_repo_with_two_commits(&frontend);
    let backend_target = rev(&backend, "HEAD~1");
    let frontend_target = rev(&frontend, "HEAD~1");

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "HEAD~1"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Unstaged changes after reset:")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(head(&backend), backend_target);
    assert_eq!(head(&frontend), frontend_target);
}

#[test]
fn reset_without_child_repositories_succeeds_quietly()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "HEAD~1"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn reset_supports_optional_commit_and_each_mode()
{
    for mode in ["--soft", "--mixed", "--hard", "--merge", "--keep"]
    {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        init_vmr(tmp.path());
        let backend = tmp.path().join("backend");
        let frontend = tmp.path().join("frontend");
        setup_repo_with_two_commits(&backend);
        setup_repo_with_two_commits(&frontend);
        let backend_target = rev(&backend, "HEAD~1");
        let frontend_target = rev(&frontend, "HEAD~1");

        git_vmr()
            .current_dir(tmp.path())
            .args(["reset", mode, "HEAD~1"])
            .assert()
            .success()
            .stderr(predicate::str::is_empty());

        assert_eq!(head(&backend), backend_target);
        assert_eq!(head(&frontend), frontend_target);
    }
}

#[test]
fn reset_without_commit_lets_git_use_default_target()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_two_commits(&backend);
    setup_repo_with_two_commits(&frontend);
    fs::write(backend.join("README.md"), "changed\n")
        .expect("failed to dirty backend");
    fs::write(frontend.join("README.md"), "changed\n")
        .expect("failed to dirty frontend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "--hard"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("HEAD is now at")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());

    assert!(status_short(&backend).is_empty());
    assert!(status_short(&frontend).is_empty());
}

#[test]
fn reset_missing_ref_fails_only_affected_repository_and_attempts_others()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    setup_repo_with_two_commits(&backend);
    setup_repo_with_two_commits(&frontend);
    setup_repo_with_two_commits(&tools);
    git(&backend, ["tag", "release-base", "HEAD~1"]);
    git(&tools, ["tag", "release-base", "HEAD~1"]);
    let backend_target = rev(&backend, "release-base");
    let tools_target = rev(&tools, "release-base");
    let frontend_before = head(&frontend);

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "release-base"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Unstaged changes after reset:")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("tools"))
        )
        .stderr(predicate::str::contains("(frontend)"));

    assert_eq!(head(&backend), backend_target);
    assert_eq!(head(&tools), tools_target);
    assert_eq!(head(&frontend), frontend_before);
}

#[test]
fn reset_delegates_rejected_working_tree_state_to_git()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    setup_repo_with_two_commits(&backend);
    fs::write(backend.join("README.md"), "local\n")
        .expect("failed to dirty backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "--keep", "HEAD~1"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));
}

#[test]
fn reset_successes_are_not_rolled_back_after_another_repository_fails()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_two_commits(&backend);
    setup_repo_with_two_commits(&frontend);
    git(&backend, ["tag", "release-base", "HEAD~1"]);
    let backend_target = rev(&backend, "release-base");

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "release-base"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Unstaged changes after reset: (backend)"
        ))
        .stderr(predicate::str::contains("(frontend)"));

    assert_eq!(head(&backend), backend_target);
}

#[test]
fn reset_reports_repository_suffixes_and_orders_failures_by_repository_name()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let alpha = tmp.path().join("alpha");
    let beta = tmp.path().join("beta");
    let zeta = tmp.path().join("zeta");
    setup_repo_with_two_commits(&alpha);
    setup_repo_with_two_commits(&beta);
    setup_repo_with_two_commits(&zeta);
    git(&beta, ["tag", "release-base", "HEAD~1"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["reset", "--hard", "release-base"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("HEAD is now at").and(predicate::str::contains("(beta)")))
        .stderr(predicate::str::starts_with(
            "fatal: ambiguous argument 'release-base': unknown revision or path not in the working tree. (alpha, zeta)\n"
        ));
}

#[test]
fn reset_uses_nested_working_dir_and_global_c_option_for_discovery()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path());
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    setup_repo_with_two_commits(&backend);
    setup_repo_with_two_commits(&frontend);
    let backend_target = rev(&backend, "HEAD~1");
    let frontend_target = rev(&frontend, "HEAD~1");

    git_vmr()
        .current_dir(&frontend)
        .args(["reset", "HEAD~1"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Unstaged changes after reset:")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());
    assert_eq!(head(&backend), backend_target);
    assert_eq!(head(&frontend), frontend_target);

    write_commit(&backend, "README.md", "third\n", "third");
    write_commit(&frontend, "README.md", "third\n", "third");
    let backend_target = rev(&backend, "HEAD~1");
    let frontend_target = rev(&frontend, "HEAD~1");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["reset", "HEAD~1"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Unstaged changes after reset:")
                .and(predicate::str::contains("backend"))
                .and(predicate::str::contains("frontend"))
        )
        .stderr(predicate::str::is_empty());
    assert_eq!(head(&backend), backend_target);
    assert_eq!(head(&frontend), frontend_target);
}
