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
    fs::write(path.join(file), "content\n").expect("failed to write file");
    git(path, ["add", file]);
    git(path, ["commit", "-m", "initial"]);
}

fn staged_names(repo: &Path) -> String
{
    git_output(repo, ["diff", "--cached", "--name-only"])
}

#[test]
fn add_stages_file_from_vmr_root()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    fs::write(backend.join("src.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "backend/src.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert_eq!(staged_names(&backend), "src.rs\n");
}

#[test]
fn add_stages_paths_across_multiple_repositories()
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
    fs::write(frontend.join("app.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "backend/src.rs", "frontend/app.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert_eq!(staged_names(&backend), "src.rs\n");
    assert_eq!(staged_names(&frontend), "app.rs\n");
}

#[test]
fn add_stages_from_child_repo_and_with_working_dir_argument()
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
    fs::write(backend.join("lib.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(&frontend)
        .args(["add", "../backend/src.rs"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .arg("-C")
        .arg(&frontend)
        .args(["add", "../backend/lib.rs"])
        .assert()
        .success();

    assert_eq!(staged_names(&backend), "lib.rs\nsrc.rs\n");
}

#[test]
fn add_dot_stages_all_repos_from_root_and_current_subtree_from_child()
{
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
    fs::create_dir(frontend.join("src")).expect("failed to create src dir");
    fs::write(backend.join("root.rs"), "changed\n")
        .expect("failed to write file");
    fs::write(frontend.join("src/app.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "."])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert_eq!(staged_names(&backend), "root.rs\n");
    assert_eq!(staged_names(&frontend), "src/app.rs\n");

    git(&backend, ["reset"]);
    git(&frontend, ["reset"]);
    fs::write(frontend.join("src/other.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(frontend.join("src"))
        .args(["add", "."])
        .assert()
        .success();

    assert_eq!(staged_names(&backend), "");
    assert_eq!(staged_names(&frontend), "src/app.rs\nsrc/other.rs\n");
}

#[test]
fn add_stages_untracked_and_deleted_files()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "old.rs");
    fs::write(backend.join("new.rs"), "changed\n")
        .expect("failed to write file");
    fs::remove_file(backend.join("old.rs")).expect("failed to delete file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "backend/new.rs", "backend/old.rs"])
        .assert()
        .success();

    let diff = git_output(&backend, ["diff", "--cached", "--name-status"]);
    assert!(diff.contains("A\tnew.rs"), "unexpected diff: {diff}");
    assert!(diff.contains("D\told.rs"), "unexpected diff: {diff}");
}

#[test]
fn add_rejects_invalid_paths_before_staging_any_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    fs::write(tmp.path().join("README.md"), "vmr\n")
        .expect("failed to write root file");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    fs::write(backend.join("src.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "backend/src.rs", "docs/readme.md"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("child Git repository"));
    assert_eq!(staged_names(&backend), "");

    for invalid in [".gitvmr/config", "README.md", "../outside.txt"]
    {
        git_vmr()
            .current_dir(tmp.path())
            .args(["add", invalid])
            .assert()
            .failure()
            .stdout(predicate::str::is_empty());
    }

    assert_eq!(staged_names(&backend), "");
}
