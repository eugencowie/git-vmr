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
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
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
    String::from_utf8(output.stdout).expect("git output should be utf8")
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
    if let Some(parent) = path.join(file).parent()
    {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path.join(file), "content\n").expect("failed to write file");
    git(path, ["add", file]);
    git(path, ["commit", "-m", "initial"]);
}

fn staged_names(repo: &Path) -> String
{
    git_output(repo, ["diff", "--cached", "--name-only"])
}

fn status_short(repo: &Path) -> String
{
    git_output(repo, ["status", "--short"])
}

#[test]
fn restore_discards_modified_file_from_vmr_root()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");
    fs::write(backend.join("src/main.rs"), "changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "backend/src/main.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    // Assert
    assert_eq!(
        fs::read_to_string(backend.join("src/main.rs")).unwrap(),
        "content\n"
    );
    assert_eq!(status_short(&backend), "");
}

#[test]
fn restore_restores_deleted_file_from_vmr_root()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");
    fs::remove_file(backend.join("src/main.rs"))
        .expect("failed to delete file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "backend/src/main.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(
        fs::read_to_string(backend.join("src/main.rs")).unwrap(),
        "content\n"
    );
    assert_eq!(status_short(&backend), "");
}

#[test]
fn restore_restores_paths_across_multiple_repositories()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/main.rs");
    commit_file(&frontend, "src/app.rs");
    fs::write(backend.join("src/main.rs"), "backend changed\n")
        .expect("failed to modify file");
    fs::write(frontend.join("src/app.rs"), "frontend changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "backend/src/main.rs", "frontend/src/app.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(status_short(&backend), "");
    assert_eq!(status_short(&frontend), "");
}

#[test]
fn restore_staged_unstages_without_discarding_worktree_modifications()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");
    fs::write(backend.join("src/main.rs"), "changed\n")
        .expect("failed to modify file");
    git(&backend, ["add", "src/main.rs"]);

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "--staged", "backend/src/main.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(staged_names(&backend), "");
    assert_eq!(
        fs::read_to_string(backend.join("src/main.rs")).unwrap(),
        "changed\n"
    );
    assert_eq!(status_short(&backend), " M src/main.rs\n");
}

#[test]
fn restore_worktree_restores_worktree_changes()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");
    fs::write(backend.join("src/main.rs"), "changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "--worktree", "backend/src/main.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(status_short(&backend), "");
    assert_eq!(
        fs::read_to_string(backend.join("src/main.rs")).unwrap(),
        "content\n"
    );
}

#[test]
fn restore_staged_and_worktree_restores_both_targets()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");
    fs::write(backend.join("src/main.rs"), "staged\n")
        .expect("failed to modify file");
    git(&backend, ["add", "src/main.rs"]);
    fs::write(backend.join("src/main.rs"), "worktree\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "--worktree", "--staged", "backend/src/main.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(status_short(&backend), "");
    assert_eq!(
        fs::read_to_string(backend.join("src/main.rs")).unwrap(),
        "content\n"
    );
}

#[test]
fn restore_interprets_paths_from_child_repo_and_working_dir_argument()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/main.rs");
    commit_file(&backend, "src/lib.rs");
    commit_file(&frontend, "README.md");
    fs::write(backend.join("src/main.rs"), "changed\n")
        .expect("failed to modify file");
    fs::write(backend.join("src/lib.rs"), "changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(&frontend)
        .args(["restore", "../backend/src/main.rs"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .arg("-C")
        .arg(&frontend)
        .args(["restore", "../backend/src/lib.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(status_short(&backend), "");
}

#[test]
fn restore_dot_restores_all_repos_from_root_and_current_subtree_from_child()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "src/app.rs");
    fs::write(backend.join("README.md"), "changed\n")
        .expect("failed to modify file");
    fs::write(frontend.join("src/app.rs"), "changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr().current_dir(tmp.path()).args(["restore", "."]).assert().success();

    // Assert
    assert_eq!(status_short(&backend), "");
    assert_eq!(status_short(&frontend), "");

    // Arrange
    fs::write(backend.join("README.md"), "changed\n")
        .expect("failed to modify file");
    fs::write(frontend.join("src/app.rs"), "changed\n")
        .expect("failed to modify file");
    git(&backend, ["add", "README.md"]);
    git(&frontend, ["add", "src/app.rs"]);

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "--staged", "."])
        .assert()
        .success();

    // Assert
    assert_eq!(staged_names(&backend), "");
    assert_eq!(staged_names(&frontend), "");

    // Arrange
    fs::write(frontend.join("src/app.rs"), "subtree changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(frontend.join("src"))
        .args(["restore", "."])
        .assert()
        .success();

    // Assert
    assert_eq!(status_short(&frontend), "");
}

#[test]
fn restore_rejects_invalid_paths_before_changing_any_repo()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    fs::write(tmp.path().join("README.md"), "vmr\n")
        .expect("failed to write root file");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "tracked.rs");
    fs::write(backend.join("tracked.rs"), "changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "backend/tracked.rs", "docs/readme.md"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("child Git repository"));

    // Assert
    assert_eq!(
        fs::read_to_string(backend.join("tracked.rs")).unwrap(),
        "changed\n"
    );

    // Act
    for invalid in [".gitvmr/config", "README.md", "../outside.txt"]
    {
        git_vmr()
            .current_dir(tmp.path())
            .args(["restore", invalid])
            .assert()
            .failure()
            .stdout(predicate::str::is_empty());
    }

    // Assert
    assert_eq!(
        fs::read_to_string(backend.join("tracked.rs")).unwrap(),
        "changed\n"
    );
}

#[test]
fn restore_preserves_git_failure_for_untracked_path_with_repo_context()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "tracked.rs");
    fs::write(backend.join("untracked.rs"), "content\n")
        .expect("failed to write untracked");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "backend/untracked.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("git restore failed for")
                .and(predicate::str::contains("backend"))
        );

    // Assert
    assert!(backend.join("untracked.rs").exists());
}

#[test]
fn restore_source_option_is_rejected_by_clap()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "tracked.rs");
    fs::write(backend.join("tracked.rs"), "changed\n")
        .expect("failed to modify file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["restore", "--source", "HEAD~1", "backend/tracked.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("unexpected argument '--source'"));

    // Assert
    assert_eq!(
        fs::read_to_string(backend.join("tracked.rs")).unwrap(),
        "changed\n"
    );
}
