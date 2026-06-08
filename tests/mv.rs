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

fn staged_status(repo: &Path) -> String
{
    git_output(repo, ["diff", "--cached", "--name-status"])
}

#[test]
fn mv_requires_at_least_source_and_destination_operands_and_rejects_options()
{
    git_vmr()
        .args(["mv"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    git_vmr()
        .args(["mv", "one"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    git_vmr()
        .args(["mv", "-r", "one", "two"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));

    for option in ["--dry-run", "-k", "--force"]
    {
        git_vmr()
            .args(["mv", option, "one", "two"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("unexpected argument"));
    }
}

#[test]
fn mv_moves_tracked_file_and_directory_within_one_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/old.rs");
    commit_file(&backend, "old-dir/a.rs");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/old.rs", "backend/src/new.rs"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/old-dir", "backend/new-dir"])
        .assert()
        .success();

    assert!(!backend.join("src/old.rs").exists());
    assert!(backend.join("src/new.rs").exists());
    assert!(!backend.join("old-dir/a.rs").exists());
    assert!(backend.join("new-dir/a.rs").exists());
    let status = staged_status(&backend);
    assert!(
        status.contains("R100\tsrc/old.rs\tsrc/new.rs"),
        "unexpected status: {status}"
    );
    assert!(
        status.contains("R100\told-dir/a.rs\tnew-dir/a.rs"),
        "unexpected status: {status}"
    );
}

#[test]
fn mv_moves_tracked_file_and_directory_between_repos()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/shared.rs");
    commit_file(&backend, "shared/a.rs");
    commit_file(&frontend, "README.md");
    fs::create_dir(frontend.join("src"))
        .expect("failed to create frontend src");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/shared.rs", "frontend/src/shared.rs"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/shared", "frontend/shared"])
        .assert()
        .success();

    assert!(!backend.join("src/shared.rs").exists());
    assert!(frontend.join("src/shared.rs").exists());
    assert!(!backend.join("shared/a.rs").exists());
    assert!(frontend.join("shared/a.rs").exists());
    let backend_status = staged_status(&backend);
    assert!(
        backend_status.contains("D\tsrc/shared.rs"),
        "unexpected status: {backend_status}"
    );
    assert!(
        backend_status.contains("D\tshared/a.rs"),
        "unexpected status: {backend_status}"
    );
    let frontend_status = staged_status(&frontend);
    assert!(
        frontend_status.contains("A\tsrc/shared.rs"),
        "unexpected status: {frontend_status}"
    );
    assert!(
        frontend_status.contains("A\tshared/a.rs"),
        "unexpected status: {frontend_status}"
    );
}

#[test]
fn mv_uses_child_working_dir_and_working_dir_argument()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/old.rs");
    commit_file(&backend, "src/lib.rs");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&frontend)
        .args(["mv", "../backend/src/old.rs", "../backend/src/new.rs"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .arg("-C")
        .arg(&frontend)
        .args(["mv", "../backend/src/lib.rs", "../backend/src/library.rs"])
        .assert()
        .success();

    let status = staged_status(&backend);
    assert!(
        status.contains("R100\tsrc/old.rs\tsrc/new.rs"),
        "unexpected status: {status}"
    );
    assert!(
        status.contains("R100\tsrc/lib.rs\tsrc/library.rs"),
        "unexpected status: {status}"
    );
}

#[test]
fn mv_supports_destination_directory_semantics()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/main.rs");
    commit_file(&backend, "src/shared.rs");
    commit_file(&frontend, "README.md");
    fs::create_dir(backend.join("archive")).expect("failed to create archive");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/main.rs", "backend/archive"])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/shared.rs", "frontend"])
        .assert()
        .success();

    assert!(backend.join("archive/main.rs").exists());
    assert!(frontend.join("shared.rs").exists());
}

#[test]
fn mv_moves_multiple_files_within_one_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/a.rs");
    commit_file(&backend, "src/b.rs");
    fs::create_dir(backend.join("archive")).expect("failed to create archive");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/a.rs", "backend/src/b.rs", "backend/archive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(!backend.join("src/a.rs").exists());
    assert!(!backend.join("src/b.rs").exists());
    assert!(backend.join("archive/a.rs").exists());
    assert!(backend.join("archive/b.rs").exists());
    let status = staged_status(&backend);
    assert!(
        status.contains("R100\tsrc/a.rs\tarchive/a.rs"),
        "unexpected status: {status}"
    );
    assert!(
        status.contains("R100\tsrc/b.rs\tarchive/b.rs"),
        "unexpected status: {status}"
    );
}

#[test]
fn mv_moves_multiple_files_from_one_repo_into_another_repo_root()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/a.rs");
    commit_file(&backend, "src/b.rs");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/a.rs", "backend/src/b.rs", "frontend"])
        .assert()
        .success();

    assert!(!backend.join("src/a.rs").exists());
    assert!(!backend.join("src/b.rs").exists());
    assert!(frontend.join("a.rs").exists());
    assert!(frontend.join("b.rs").exists());
    let backend_status = staged_status(&backend);
    assert!(
        backend_status.contains("D\tsrc/a.rs"),
        "unexpected status: {backend_status}"
    );
    assert!(
        backend_status.contains("D\tsrc/b.rs"),
        "unexpected status: {backend_status}"
    );
    let frontend_status = staged_status(&frontend);
    assert!(
        frontend_status.contains("A\ta.rs"),
        "unexpected status: {frontend_status}"
    );
    assert!(
        frontend_status.contains("A\tb.rs"),
        "unexpected status: {frontend_status}"
    );
}

#[test]
fn mv_moves_multiple_files_from_multiple_repos_into_one_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "src/a.rs");
    commit_file(&frontend, "src/b.rs");
    commit_file(&tools, "README.md");
    fs::create_dir(tools.join("archive")).expect("failed to create archive");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/a.rs", "frontend/src/b.rs", "tools/archive"])
        .assert()
        .success();

    assert!(!backend.join("src/a.rs").exists());
    assert!(!frontend.join("src/b.rs").exists());
    assert!(tools.join("archive/a.rs").exists());
    assert!(tools.join("archive/b.rs").exists());
    assert!(staged_status(&backend).contains("D\tsrc/a.rs"));
    assert!(staged_status(&frontend).contains("D\tsrc/b.rs"));
    let tools_status = staged_status(&tools);
    assert!(tools_status.contains("A\tarchive/a.rs"));
    assert!(tools_status.contains("A\tarchive/b.rs"));
}

#[test]
fn mv_multi_source_uses_child_working_dir_and_working_dir_argument()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "src/a.rs");
    commit_file(&backend, "src/b.rs");
    commit_file(&backend, "src/c.rs");
    commit_file(&backend, "src/d.rs");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    fs::create_dir(frontend.join("archive")).expect("failed to create archive");

    git_vmr()
        .current_dir(&tools)
        .args([
            "mv",
            "../backend/src/a.rs",
            "../backend/src/b.rs",
            "../frontend/archive"
        ])
        .assert()
        .success();
    git_vmr()
        .current_dir(tmp.path())
        .arg("-C")
        .arg(&tools)
        .args([
            "mv",
            "../backend/src/c.rs",
            "../backend/src/d.rs",
            "../frontend/archive"
        ])
        .assert()
        .success();

    assert!(frontend.join("archive/a.rs").exists());
    assert!(frontend.join("archive/b.rs").exists());
    assert!(frontend.join("archive/c.rs").exists());
    assert!(frontend.join("archive/d.rs").exists());
}

#[test]
fn mv_rejects_invalid_ownership_before_moving()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    fs::write(tmp.path().join("README.md"), "vmr\n")
        .expect("failed to write root file");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "src/main.rs");

    for (source, destination, message) in [
        ("../outside.txt", "backend/outside.txt", "outside virtual monorepo"),
        ("backend/src/main.rs", "../outside.txt", "outside virtual monorepo"),
        ("backend/src/main.rs", ".gitvmr/main.rs", "metadata"),
        ("backend/src/main.rs", "docs/main.rs", "child Git repository"),
        ("backend/src/main.rs", "README.md", "child Git repository"),
        (".", "backend/all", "VMR root aggregate")
    ]
    {
        git_vmr()
            .current_dir(tmp.path())
            .args(["mv", source, destination])
            .assert()
            .failure()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(message));
        assert!(backend.join("src/main.rs").exists());
        assert_eq!(staged_status(&backend), "");
    }
}

#[test]
fn mv_rejects_cross_repo_untracked_source_before_moving()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "tracked.rs");
    commit_file(&frontend, "README.md");
    fs::write(backend.join("untracked.rs"), "content\n")
        .expect("failed to write untracked");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/untracked.rs", "frontend/untracked.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("source path is not tracked")
                .and(predicate::str::contains("backend"))
        );

    assert!(backend.join("untracked.rs").exists());
    assert!(!frontend.join("untracked.rs").exists());
    assert_eq!(staged_status(&backend), "");
    assert_eq!(staged_status(&frontend), "");
}

#[test]
fn mv_rejects_cross_repo_destination_conflicts_and_missing_parents_before_moving()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "src/main.rs");
    commit_file(&backend, "src/lib.rs");
    commit_file(&frontend, "existing.rs");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/main.rs", "frontend/existing.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("error: destination")
                .and(predicate::str::contains("already exists"))
                .and(
                    predicate::str::contains("fatal: error: destination").not()
                )
        );
    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/src/lib.rs", "frontend/missing/lib.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("error: destination parent").and(
            predicate::str::contains("fatal: error: destination parent").not()
        ));

    assert!(backend.join("src/main.rs").exists());
    assert!(backend.join("src/lib.rs").exists());
    assert_eq!(staged_status(&backend), "");
    assert_eq!(staged_status(&frontend), "");
}

#[test]
fn mv_rejects_multi_source_preflight_failures_before_moving()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    let tools = tmp.path().join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "src/a.rs");
    commit_file(&backend, "src/b.rs");
    commit_file(&frontend, "src/a.rs");
    commit_file(&frontend, "src/c.rs");
    commit_file(&tools, "README.md");
    fs::create_dir(tools.join("archive")).expect("failed to create archive");
    commit_file(&tools, "archive/existing.rs");
    fs::write(frontend.join("untracked.rs"), "content\n")
        .expect("failed to write untracked");

    for (args, message) in [
        (
            vec!["mv", "backend/src/a.rs", "backend/src/b.rs", "tools/missing"],
            "not an existing directory"
        ),
        (
            vec![
                "mv",
                "backend/src/a.rs",
                "backend/src/b.rs",
                "tools/archive/existing.rs",
            ],
            "not an existing directory"
        ),
        (
            vec![
                "mv",
                "backend/src/a.rs",
                "frontend/src/a.rs",
                "tools/archive",
            ],
            "duplicate destination"
        ),
        (
            vec![
                "mv",
                "tools/archive/existing.rs",
                "frontend/src/c.rs",
                "tools/archive",
            ],
            "already exists"
        ),
        (
            vec!["mv", "backend/src/a.rs", "docs/b.rs", "tools/archive"],
            "child Git repository"
        ),
        (
            vec![
                "mv",
                "backend/src/a.rs",
                "frontend/untracked.rs",
                "tools/archive",
            ],
            "source path is not tracked"
        )
    ]
    {
        git_vmr()
            .current_dir(tmp.path())
            .args(args)
            .assert()
            .failure()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(message));

        assert!(backend.join("src/a.rs").exists());
        assert!(backend.join("src/b.rs").exists());
        assert!(frontend.join("src/a.rs").exists());
        assert!(frontend.join("src/c.rs").exists());
        assert!(tools.join("archive/existing.rs").exists());
        assert_eq!(staged_status(&backend), "");
        assert_eq!(staged_status(&frontend), "");
        assert_eq!(staged_status(&tools), "");
    }
}

#[test]
fn mv_same_repo_git_failure_reports_repo_context()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "tracked.rs");
    fs::write(backend.join("untracked.rs"), "content\n")
        .expect("failed to write untracked");

    git_vmr()
        .current_dir(tmp.path())
        .args(["mv", "backend/untracked.rs", "backend/moved.rs"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("git mv failed for")
                .and(predicate::str::contains("backend"))
        );

    assert!(backend.join("untracked.rs").exists());
    assert_eq!(staged_status(&backend), "");
}
