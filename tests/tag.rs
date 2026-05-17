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
        .args(["tag", "v1.0.0"])
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
