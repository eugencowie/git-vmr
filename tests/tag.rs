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

fn tag_exists(path: &Path, tag: &str) -> bool
{
    std::process::Command::new("git")
        .args(["rev-parse", "--verify", tag])
        .current_dir(path)
        .output()
        .expect("failed to run git")
        .status
        .success()
}

#[test]
fn tag_lists_local_tags_across_child_repositories()
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
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    git(&backend, ["tag", "v1.0.0"]);
    git(&frontend, ["tag", "v1.0.0"]);
    git(&tools, ["tag", "v1.0.0"]);
    git(&backend, ["tag", "v1.1.0"]);
    git(&frontend, ["tag", "v1.1.0"]);
    git(&tools, ["tag", "v2.0.0-rc1"]);

    git_vmr()
        .current_dir(tmp.path())
        .arg("tag")
        .assert()
        .success()
        .stdout(predicate::eq(
            "v1.0.0\nv1.1.0 (backend, frontend)\nv2.0.0-rc1 (tools)\n"
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn tag_skips_non_git_children_and_empty_vmr_outputs_nothing()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("tag")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    git(&backend, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .arg("tag")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("v1.0.0")
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn tag_with_child_repositories_and_no_local_tags_outputs_nothing()
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

    git_vmr()
        .current_dir(tmp.path())
        .arg("tag")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn tag_uses_nested_working_dir_and_global_c_option_for_discovery()
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
    git(&backend, ["tag", "backend-only"]);
    git(&frontend, ["tag", "frontend-only"]);

    git_vmr()
        .current_dir(&frontend)
        .arg("tag")
        .assert()
        .success()
        .stdout(predicate::eq(
            "backend-only (backend)\nfrontend-only (frontend)\n"
        ))
        .stderr(predicate::str::is_empty());

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("tag")
        .assert()
        .success()
        .stdout(predicate::eq(
            "backend-only (backend)\nfrontend-only (frontend)\n"
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn tag_fails_on_corrupted_git_dir()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let repo = tmp.path().join("broken");
    fs::create_dir(&repo).expect("failed to create repo");
    fs::write(repo.join(".git"), "not a gitfile\n")
        .expect("failed to corrupt git dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("tag")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to read tag information"));
}

#[test]
fn tag_rejects_unsupported_arguments()
{
    git_vmr()
        .args(["tag", "v1.0.0", "HEAD~1"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());

    git_vmr()
        .args(["tag", "-a", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());

    git_vmr()
        .args(["tag", "-f", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());

    git_vmr()
        .args(["tag", "-d", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());

    git_vmr()
        .args(["tag", "--contains", "HEAD"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());
}

#[test]
fn tag_creates_lightweight_tag_in_every_child_repository_with_no_output()
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

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "v1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(tag_exists(&backend, "v1.0.0"));
    assert!(tag_exists(&frontend, "v1.0.0"));
}

#[test]
fn tag_create_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "v1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(tag_exists(&backend, "v1.0.0"));
}

#[test]
fn tag_create_partial_failure_does_not_stop_other_repositories()
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
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    git(&backend, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "fatal: tag 'v1.0.0' already exists (backend)"
        ));

    assert!(tag_exists(&frontend, "v1.0.0"));
    assert!(tag_exists(&tools, "v1.0.0"));
}

#[test]
fn tag_create_reports_failures_with_repository_suffixes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    git(&backend, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: tag 'v1.0.0' already exists (backend)"
            )
            .and(predicate::str::contains("fatal: git tag failed").not())
        );
}

#[test]
fn tag_create_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let alpha = tmp.path().join("alpha");
    let zeta = tmp.path().join("zeta");
    init_repo(&alpha);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&zeta, "README.md");
    git(&alpha, ["tag", "v1.0.0"]);
    git(&zeta, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: tag 'v1.0.0' already exists (alpha)\n\
             fatal: tag 'v1.0.0' already exists (zeta)\n"
        ));
}

#[test]
fn tag_delete_deletes_tag_in_every_child_repository()
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
    git(&backend, ["tag", "v1.0.0"]);
    git(&frontend, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "-d", "v1.0.0"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deleted tag 'v1.0.0'")
                .and(predicate::str::contains("(backend)"))
                .and(predicate::str::contains("(frontend)"))
        )
        .stderr(predicate::str::is_empty());

    assert!(!tag_exists(&backend, "v1.0.0"));
    assert!(!tag_exists(&frontend, "v1.0.0"));
}

#[test]
fn tag_long_delete_deletes_tag_in_every_child_repository()
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
    git(&backend, ["tag", "v1.0.0"]);
    git(&frontend, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "--delete", "v1.0.0"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deleted tag 'v1.0.0'")
                .and(predicate::str::contains("(backend)"))
                .and(predicate::str::contains("(frontend)"))
        )
        .stderr(predicate::str::is_empty());

    assert!(!tag_exists(&backend, "v1.0.0"));
    assert!(!tag_exists(&frontend, "v1.0.0"));
}

#[test]
fn tag_delete_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create docs dir");
    let backend = tmp.path().join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");
    git(&backend, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "-d", "v1.0.0"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deleted tag 'v1.0.0'")
                .and(predicate::str::contains("(backend)"))
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(!tag_exists(&backend, "v1.0.0"));
}

#[test]
fn tag_delete_partial_failure_does_not_stop_other_repositories()
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
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    git(&frontend, ["tag", "v1.0.0"]);
    git(&tools, ["tag", "v1.0.0"]);

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "-d", "v1.0.0"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("Deleted tag 'v1.0.0'")
                .and(predicate::str::contains("(frontend)"))
                .and(predicate::str::contains("(tools)"))
        )
        .stderr(predicate::str::contains(
            "error: tag 'v1.0.0' not found. (backend)"
        ));

    assert!(!tag_exists(&frontend, "v1.0.0"));
    assert!(!tag_exists(&tools, "v1.0.0"));
}

#[test]
fn tag_delete_reports_failures_with_repository_suffixes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let backend = tmp.path().join("backend");
    let tools = tmp.path().join("tools");
    init_repo(&backend);
    init_repo(&tools);
    commit_file(&backend, "README.md");
    commit_file(&tools, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "-d", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "error: tag 'v1.0.0' not found. (backend)"
            )
            .and(predicate::str::contains(
                "error: tag 'v1.0.0' not found. (tools)"
            ))
            .and(predicate::str::contains("fatal: git tag failed").not())
        );
}

#[test]
fn tag_delete_reports_multiple_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    let alpha = tmp.path().join("alpha");
    let zeta = tmp.path().join("zeta");
    init_repo(&alpha);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&zeta, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .args(["tag", "-d", "v1.0.0"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: error: tag 'v1.0.0' not found. (alpha)\n\
             fatal: error: tag 'v1.0.0' not found. (zeta)\n"
        ));
}
