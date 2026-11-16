mod common;

use common::{commit_file, git, git_vmr, init_repo};
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Makes `remote/<name>` a repo with one commit for each of `names`, to
/// clone child repos from.
fn child_sources(remote: &Path, names: &[&str])
{
    // Make each repo with one commit
    for name in names
    {
        let repo = remote.join(name);
        init_repo(&repo);
        commit_file(&repo, "README.md");
    }
}

/// Makes `remote/vmr` a root repo to clone from, its committed manifest
/// declaring `repos` as (path, URL) pairs and its `.gitignore` ignoring
/// them.
fn root_source(remote: &Path, repos: &[(&str, &str)]) -> PathBuf
{
    // Make a root repo
    let root = remote.join("vmr");
    init_repo(&root);

    // Declare child repos and commit the declarations
    let manifest = repos
        .iter()
        .map(|(path, url)| {
            format!("[repo.{path}]\npath = \"{path}\"\nurl = '{url}'\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let gitignore =
        repos.iter().map(|(path, _)| format!("/{path}/\n")).collect::<String>();
    fs::write(root.join(".gitvmr"), manifest)
        .expect("failed to write manifest");
    fs::write(root.join(".gitignore"), gitignore)
        .expect("failed to write .gitignore");
    git(&root, ["add", ".gitvmr", ".gitignore"]);
    git(&root, ["commit", "--quiet", "-m", "Declare child repos"]);

    root
}

/// A remote holding a root repo that declares `backend` by a relative URL
/// and `frontend` by an absolute one, and both child repos.
fn remote_vmr(tmp: &Path) -> PathBuf
{
    // Make child repo sources
    let remote = tmp.join("remote");
    fs::create_dir(&remote).expect("failed to create remote dir");
    child_sources(&remote, &["backend", "frontend"]);

    // Make a root repo source declaring them
    let frontend = remote.join("frontend").display().to_string();
    root_source(&remote, &[("backend", "../backend"), ("frontend", &frontend)])
}

/// An empty directory to clone into.
fn clone_dir(tmp: &Path) -> PathBuf
{
    let dir = tmp.join("clones");
    fs::create_dir(&dir).expect("failed to create clone dir");
    dir
}

#[test]
fn clone_clones_the_root_repo_then_every_declared_child_repo()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = remote_vmr(tmp.path());
    let clones = clone_dir(tmp.path());

    // Act + Assert: clone
    git_vmr()
        .current_dir(&clones)
        .arg("clone")
        .arg(&source)
        .arg("copy")
        .assert()
        .success()
        .stdout("Cloned (backend, frontend)\n")
        .stderr(predicate::str::contains("Cloning into 'copy'"));
    let copy = clones.join("copy");
    assert!(copy.join(".gitvmr").is_file());
    assert!(copy.join("backend/README.md").is_file());
    assert!(copy.join("frontend/README.md").is_file());

    // Act + Assert: the clone is a VMR with nothing to warn about
    git_vmr()
        .current_dir(&copy)
        .arg("status")
        .assert()
        .success()
        .stdout("On branch master\n\nnothing to commit, working tree clean\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn clone_uses_git_inferred_destination()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = remote_vmr(tmp.path());
    let clones = clone_dir(tmp.path());

    // Act
    git_vmr()
        .current_dir(&clones)
        .arg("clone")
        .arg(&source)
        .assert()
        .success()
        .stderr(predicate::str::contains("Cloning into 'vmr'"));

    // Assert
    assert!(clones.join("vmr/.gitvmr").is_file());
    assert!(clones.join("vmr/backend/README.md").is_file());
}

#[test]
fn clone_destination_is_relative_to_working_dir_argument()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = remote_vmr(tmp.path());
    let clones = clone_dir(tmp.path());
    let base = clones.join("base");
    fs::create_dir(&base).expect("failed to create base");

    // Act
    git_vmr()
        .current_dir(&clones)
        .arg("-C")
        .arg(&base)
        .arg("clone")
        .arg(&source)
        .arg("copy")
        .assert()
        .success()
        .stderr(predicate::str::contains("Cloning into 'copy'"));

    // Assert
    assert!(base.join("copy/frontend/README.md").is_file());
    assert!(!clones.join("copy").exists());
}

#[test]
fn clone_keeps_a_root_repo_without_a_manifest_and_fails()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let source = tmp.path().join("project");
    init_repo(&source);
    commit_file(&source, "README.md");
    let clones = clone_dir(tmp.path());

    // Act
    git_vmr()
        .current_dir(&clones)
        .arg("clone")
        .arg(&source)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::ends_with(
            "fatal: cloned 'project', but it has no .gitvmr manifest\n\
             hint: the root repo's default branch may not have one\n"
        ));

    // Assert
    assert!(clones.join("project/README.md").is_file());
}

#[test]
fn clone_failed_git_process_exits_non_zero()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let clones = clone_dir(tmp.path());

    // Act
    git_vmr()
        .current_dir(&clones)
        .arg("clone")
        .arg("foo")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr("fatal: repository 'foo' does not exist\n");

    // Assert
    assert!(!clones.join("foo").exists());
}

#[test]
fn clone_keeps_a_partial_vmr_that_clone_without_a_repository_completes()
{
    // Arrange: `api` is declared but has no source yet
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let remote = tmp.path().join("remote");
    fs::create_dir(&remote).expect("failed to create remote dir");
    child_sources(&remote, &["backend"]);
    let source =
        root_source(&remote, &[("api", "../api"), ("backend", "../backend")]);
    let clones = clone_dir(tmp.path());
    let copy = clones.join("copy");

    // Act + Assert: the failed child clone fails the command
    git_vmr()
        .current_dir(&clones)
        .arg("clone")
        .arg(&source)
        .arg("copy")
        .assert()
        .failure()
        .stdout("Cloned (backend)\n")
        .stderr(predicate::str::contains("does not exist (api)\n"));
    assert!(copy.join("backend/README.md").is_file());
    assert!(!copy.join("api").exists());

    // Act + Assert: once `api` has a source, clone fills the gap
    child_sources(&remote, &["api"]);
    git_vmr()
        .current_dir(copy.join("backend"))
        .arg("clone")
        .assert()
        .success()
        .stdout("Cloned (api)\n")
        .stderr(predicate::str::is_empty());
    assert!(copy.join("api/README.md").is_file());
}

#[test]
fn clone_without_a_repository_fails_outside_a_vmr()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    // Act + Assert
    git_vmr()
        .current_dir(tmp.path())
        .arg("clone")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: not a virtual monorepo"));
}
