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
    assert_eq!(staged_names(&backend), "src/main.rs\n");
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
    assert_eq!(staged_names(&backend), "src/main.rs\n");
    assert_eq!(staged_names(&frontend), "src/app.rs\n");
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
    assert_eq!(staged_names(&backend), "src/lib.rs\nsrc/main.rs\n");
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
    assert_eq!(staged_names(&backend), "src/lib.rs\nsrc/main.rs\n");
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
    assert_eq!(staged_names(&backend), "README.md\n");
    assert_eq!(staged_names(&frontend), "README.md\n");
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
