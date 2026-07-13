mod common;

use common::{commit_file, git_output, git_vmr, init_repo};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn staged_names(repo: &Path) -> String
{
    git_output(repo, ["diff", "--cached", "--name-only"])
}

fn unstaged_names(repo: &Path) -> String
{
    git_output(repo, ["diff", "--name-only"])
}

#[test]
fn rm_removes_tracked_file_from_vmr_root()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "backend/src/main.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    // Assert
    assert!(!backend.join("src/main.rs").exists());
    assert_eq!(staged_names(&backend), "src/main.rs");
}

#[test]
fn rm_removes_paths_across_multiple_repositories()
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

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "backend/src/main.rs", "frontend/src/app.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(staged_names(&backend), "src/main.rs");
    assert_eq!(staged_names(&frontend), "src/app.rs");
}

#[test]
fn rm_removes_from_child_repo_and_with_working_dir_argument()
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

    // Act
    git_vmr()
        .current_dir(&frontend)
        .args(["rm", "../backend/src/main.rs"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .arg("-C")
        .arg(&frontend)
        .args(["rm", "../backend/src/lib.rs"])
        .assert()
        .success();

    // Assert
    assert_eq!(staged_names(&backend), "src/lib.rs\nsrc/main.rs");
}

#[test]
fn rm_preserves_git_failure_for_untracked_path_with_repo_context()
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
        .args(["rm", "backend/untracked.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("git rm failed for")
                .and(predicate::str::contains("backend"))
        );

    // Assert
    assert!(backend.join("untracked.rs").exists());
    assert_eq!(staged_names(&backend), "");
}

#[test]
fn rm_recursive_removes_directories_with_short_flag()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");
    commit_file(&backend, "src/lib.rs");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "-r", "backend/src"])
        .assert()
        .success();

    // Assert
    assert_eq!(staged_names(&backend), "src/lib.rs\nsrc/main.rs");
}

#[test]
fn rm_force_removes_modified_tracked_file_with_short_and_long_flags()
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
    fs::write(backend.join("src/main.rs"), "modified\n")
        .expect("failed to modify backend file");
    fs::write(frontend.join("src/app.rs"), "modified\n")
        .expect("failed to modify frontend file");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "-f", "backend/src/main.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "--force", "frontend/src/app.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    // Assert
    assert!(!backend.join("src/main.rs").exists());
    assert!(!frontend.join("src/app.rs").exists());
    assert_eq!(staged_names(&backend), "src/main.rs");
    assert_eq!(staged_names(&frontend), "src/app.rs");
}

#[test]
fn rm_dry_run_reports_preview_without_mutating_short_and_long_flags()
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

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "-n", "backend/src/main.rs"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("rm 'src/main.rs'")
                .and(predicate::str::contains("(backend)").not())
        );
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "--dry-run", "frontend/src/app.rs"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("rm 'src/app.rs'")
                .and(predicate::str::contains("(frontend)").not())
        );

    // Assert
    assert!(backend.join("src/main.rs").exists());
    assert!(frontend.join("src/app.rs").exists());
    assert_eq!(staged_names(&backend), "");
    assert_eq!(staged_names(&frontend), "");
    assert_eq!(unstaged_names(&backend), "");
    assert_eq!(unstaged_names(&frontend), "");
}

#[test]
fn rm_cached_removes_file_and_directory_from_index_only()
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
    commit_file(&frontend, "src/lib.rs");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "--cached", "backend/src/main.rs"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "--cached", "-r", "frontend/src"])
        .assert()
        .success();

    // Assert
    assert!(backend.join("src/main.rs").exists());
    assert!(frontend.join("src/app.rs").exists());
    assert!(frontend.join("src/lib.rs").exists());
    assert_eq!(staged_names(&backend), "src/main.rs");
    assert_eq!(staged_names(&frontend), "src/app.rs\nsrc/lib.rs");
}

#[test]
fn rm_directory_without_recursive_fails_without_removing_files()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "backend/src"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("git rm failed for"));

    // Assert
    assert!(backend.join("src/main.rs").exists());
    assert_eq!(staged_names(&backend), "");
}

#[test]
fn rm_dry_run_root_requires_recursive_and_recursive_preview_does_not_mutate()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "-n", "."])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("without -r"));

    // Assert
    assert_eq!(staged_names(&backend), "");
    assert_eq!(staged_names(&frontend), "");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "-n", "-r", "."])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("rm 'README.md'")
                .and(predicate::str::contains("(backend, frontend)").not())
        );

    // Assert
    assert!(backend.join("README.md").exists());
    assert!(frontend.join("README.md").exists());
    assert_eq!(staged_names(&backend), "");
    assert_eq!(staged_names(&frontend), "");
}

#[test]
fn rm_vmr_root_requires_recursive_and_expands_to_git_children()
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
    commit_file(&frontend, "README.md");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "."])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("without -r"));

    // Assert
    assert_eq!(staged_names(&backend), "");
    assert_eq!(staged_names(&frontend), "");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "-r", "."])
        .assert()
        .success();

    // Assert
    assert_eq!(staged_names(&backend), "README.md");
    assert_eq!(staged_names(&frontend), "README.md");
    assert!(tmp.path().join("docs").exists());
}

#[test]
fn rm_rejects_invalid_paths_before_changing_any_repo()
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

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .args(["rm", "backend/tracked.rs", "docs/readme.md"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("child Git repository"));

    // Assert
    assert!(backend.join("tracked.rs").exists());
    assert_eq!(staged_names(&backend), "");

    // Act
    for invalid in [".gitvmr/config", "README.md", "../outside.txt"]
    {
        git_vmr()
            .current_dir(tmp.path())
            .args(["rm", invalid])
            .assert()
            .failure()
            .stdout(predicate::str::is_empty());
    }

    // Assert
    assert!(backend.join("tracked.rs").exists());
    assert_eq!(staged_names(&backend), "");
}

#[test]
fn rm_flags_do_not_bypass_vmr_path_validation()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    fs::write(tmp.path().join("docs/readme.md"), "docs\n")
        .expect("failed to write docs file");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "tracked.rs");

    // Act + Assert
    for args in [
        ["rm", "--force", ".gitvmr/config"],
        ["rm", "--dry-run", "docs/readme.md"],
        ["rm", "--cached", "../outside.txt"]
    ]
    {
        git_vmr()
            .current_dir(tmp.path())
            .args(args)
            .assert()
            .failure()
            .stdout(predicate::str::is_empty());
        assert!(backend.join("tracked.rs").exists());
        assert_eq!(staged_names(&backend), "");
    }
}
