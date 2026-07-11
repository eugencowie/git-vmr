mod common;

use common::{commit_file, git, git_output, git_vmr, init_repo};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn staged_names(repo: &Path) -> String
{
    git_output(repo, ["diff", "--cached", "--name-only"])
}

fn staged_status(repo: &Path) -> String
{
    git_output(repo, ["diff", "--cached", "--name-status"])
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

    assert_eq!(staged_names(&backend), "src.rs");
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

    assert_eq!(staged_names(&backend), "src.rs");
    assert_eq!(staged_names(&frontend), "app.rs");
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

    assert_eq!(staged_names(&backend), "lib.rs\nsrc.rs");
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

    assert_eq!(staged_names(&backend), "root.rs");
    assert_eq!(staged_names(&frontend), "src/app.rs");

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
    assert_eq!(staged_names(&frontend), "src/app.rs\nsrc/other.rs");
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

fn assert_add_all_stages_every_repo(flag: &str)
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "old.rs");
    commit_file(&frontend, "app.rs");
    fs::write(backend.join("new.rs"), "new\n").expect("failed to write file");
    fs::write(backend.join("old.rs"), "changed\n")
        .expect("failed to write file");
    fs::remove_file(frontend.join("app.rs")).expect("failed to delete file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", flag])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend_diff = staged_status(&backend);
    assert!(
        backend_diff.contains("A\tnew.rs"),
        "unexpected diff: {backend_diff}"
    );
    assert!(
        backend_diff.contains("M\told.rs"),
        "unexpected diff: {backend_diff}"
    );
    assert_eq!(staged_status(&frontend), "D\tapp.rs");
}

#[test]
fn add_all_short_stages_every_repo_without_pathspecs()
{
    assert_add_all_stages_every_repo("-A");
}

#[test]
fn add_all_long_stages_every_repo_without_pathspecs()
{
    assert_add_all_stages_every_repo("--all");
}

#[test]
fn add_all_without_pathspecs_stages_every_repo_from_child()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "old.rs");
    commit_file(&frontend, "app.rs");
    fs::write(backend.join("old.rs"), "changed\n")
        .expect("failed to write file");
    fs::write(frontend.join("new.rs"), "new\n").expect("failed to write file");

    git_vmr().current_dir(&frontend).args(["add", "-A"]).assert().success();

    assert_eq!(staged_status(&backend), "M\told.rs");
    assert_eq!(staged_status(&frontend), "A\tnew.rs");
}

#[test]
fn add_all_with_pathspecs_preserves_routed_scoping()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "old.rs");
    commit_file(&frontend, "app.rs");
    fs::write(backend.join("old.rs"), "changed\n")
        .expect("failed to write file");
    fs::write(backend.join("other.rs"), "other\n")
        .expect("failed to write file");
    fs::write(frontend.join("app.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "-A", "backend/old.rs", "frontend/app.rs"])
        .assert()
        .success();

    assert_eq!(staged_names(&backend), "old.rs");
    assert_eq!(staged_names(&frontend), "app.rs");
}

fn assert_force_stages_ignored_file(flag: &str)
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    fs::write(backend.join(".gitignore"), "*.log\n")
        .expect("failed to write gitignore");
    fs::write(backend.join("generated.log"), "ignored\n")
        .expect("failed to write ignored file");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", flag, "backend/generated.log"])
        .assert()
        .success();

    assert_eq!(staged_names(&backend), "generated.log");
}

#[test]
fn add_force_short_stages_ignored_file()
{
    assert_force_stages_ignored_file("-f");
}

#[test]
fn add_force_long_stages_ignored_file()
{
    assert_force_stages_ignored_file("--force");
}

#[test]
fn add_chmod_updates_executable_bit_in_index()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "script.sh");

    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "--chmod=+x", "backend/script.sh"])
        .assert()
        .success();
    assert_eq!(
        git_output(&backend, ["diff", "--cached", "--summary"]),
        "mode change 100644 => 100755 script.sh"
    );

    git(&backend, ["commit", "-m", "chmod"]);
    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "--chmod=-x", "backend/script.sh"])
        .assert()
        .success();
    assert_eq!(
        git_output(&backend, ["diff", "--cached", "--summary"]),
        "mode change 100755 => 100644 script.sh"
    );
}

#[test]
fn add_flags_reject_invalid_paths_before_staging_any_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src.rs");

    for args in [
        vec!["add", "--force", ".gitvmr/config"],
        vec!["add", "-A", "backend/src.rs", "docs/readme.md"],
        vec!["add", "--chmod=+x", "../outside.sh"]
    ]
    {
        fs::write(backend.join("src.rs"), "changed\n")
            .expect("failed to write file");
        git_vmr()
            .current_dir(tmp.path())
            .args(args)
            .assert()
            .failure()
            .stdout(predicate::str::is_empty());
        assert_eq!(staged_names(&backend), "");
    }
}
