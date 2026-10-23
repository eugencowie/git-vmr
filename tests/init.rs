mod common;

use common::{commit_file, git, git_output, git_vmr, init_repo, shown_dir};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// A child repo at `dir/name`, with `origin` set to `url` when given.
fn child_repo(dir: &Path, name: &str, origin: Option<&str>)
{
    // Create repo
    let repo = dir.join(name);
    init_repo(&repo);

    // Set origin
    if let Some(url) = origin
    {
        git(&repo, ["remote", "add", "origin", url]);
    }
}

fn read(path: &Path) -> String
{
    fs::read_to_string(path).expect("failed to read file")
}

fn next_steps(files: &str) -> String
{
    format!(
        "\nNext, commit the manifest and add a remote:\n  \
         git add {files}\n  \
         git commit -m \"Create virtual monorepo\"\n  \
         git remote add origin <url>\n"
    )
}

#[test]
fn init_creates_root_repo_with_empty_manifest()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(format!(
            "Initialized virtual monorepo in {}\n{}",
            shown_dir(tmp.path()).display(),
            next_steps(".gitvmr")
        ))
        .stderr(predicate::str::is_empty());

    // Assert
    assert!(tmp.path().join(".git").is_dir());
    assert_eq!(read(&tmp.path().join(".gitvmr")), "");
    assert!(!tmp.path().join(".gitignore").exists());
    git_vmr().current_dir(tmp.path()).arg("status").assert().success();
}

#[test]
fn init_declares_existing_repos_from_their_origin()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    child_repo(tmp.path(), "frontend", Some("git@host:org/frontend.git"));
    child_repo(tmp.path(), "backend", Some("https://host/org/backend.git"));
    fs::create_dir(tmp.path().join("docs")).expect("failed to create dir");
    fs::create_dir_all(tmp.path().join("nested/deeper/.git"))
        .expect("failed to create nested repo");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(format!(
            "Initialized virtual monorepo in {}\n\
             Declared 'backend'\n\
             Declared 'frontend'\n{}",
            shown_dir(tmp.path()).display(),
            next_steps(".gitvmr .gitignore")
        ))
        .stderr(predicate::str::is_empty());

    // Assert
    assert_eq!(
        read(&tmp.path().join(".gitvmr")),
        "[repo.backend]\n\
         path = \"backend\"\n\
         url = \"https://host/org/backend.git\"\n\
         \n\
         [repo.frontend]\n\
         path = \"frontend\"\n\
         url = \"git@host:org/frontend.git\"\n"
    );
    assert_eq!(read(&tmp.path().join(".gitignore")), "/backend/\n/frontend/\n");
    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn init_skips_repos_without_an_origin()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    child_repo(tmp.path(), "backend", Some("../backend.git"));
    child_repo(tmp.path(), "scratch", None);

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Declared 'backend'\n"))
        .stdout(predicate::str::contains("scratch").not())
        .stderr("warning: skipped 'scratch': it has no 'origin' remote\n");

    // Assert
    assert!(!read(&tmp.path().join(".gitvmr")).contains("scratch"));
    assert_eq!(read(&tmp.path().join(".gitignore")), "/backend/\n");
}

#[test]
fn init_skips_repos_whose_origin_url_is_empty()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    child_repo(tmp.path(), "backend", None);
    git(&tmp.path().join("backend"), ["config", "remote.origin.url", ""]);

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Declared").not())
        .stderr("warning: skipped 'backend': its 'origin' URL is empty\n");

    // Assert
    git_vmr().current_dir(tmp.path()).arg("status").assert().success();
}

#[test]
fn init_errors_in_a_legacy_worktree_root()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::write(tmp.path().join(".gitvmr"), "").expect("failed to write marker");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(format!(
            "fatal: '{}' is a legacy worktree root\n\
             hint: remove its child worktrees with 'git worktree remove' in \
             each repo, then recreate it with 'git vmr worktree add'\n",
            shown_dir(tmp.path()).display()
        ));

    // Assert
    assert!(!tmp.path().join(".git").exists());
}

#[cfg(unix)]
#[test]
fn init_skips_repos_whose_name_cannot_be_declared()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    child_repo(tmp.path(), "a\\b", Some("../ab.git"));

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Declared").not())
        .stderr(
            "warning: skipped 'a\\b': path 'a\\b' must be a single directory \
             name, other than '.git' or '.gitvmr'\n"
        );

    // Assert
    git_vmr().current_dir(tmp.path()).arg("status").assert().success();
}

#[test]
fn init_skips_repos_whose_directory_the_root_repo_tracks()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_repo(tmp.path().join("root").as_path());
    let root = tmp.path().join("root");
    commit_file(&root, "vendor/lib.rs");
    child_repo(&root, "backend", Some("../backend.git"));
    git(&root.join("vendor"), ["init", "--quiet"]);
    git(&root.join("vendor"), ["remote", "add", "origin", "../vendor.git"]);

    // Act
    git_vmr()
        .current_dir(&root)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Declared 'backend'\n"))
        .stderr(
            "warning: skipped 'vendor': the root repo tracks files under it\n"
        );

    // Assert
    assert!(!read(&root.join(".gitvmr")).contains("vendor"));
}

#[test]
fn init_keeps_an_existing_root_repo_and_gitignore()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_repo(tmp.path().join("root").as_path());
    let root = tmp.path().join("root");
    fs::write(root.join(".gitignore"), "target\n/backend/").unwrap();
    commit_file(&root, "README.md");
    child_repo(&root, "backend", Some("../backend.git"));
    child_repo(&root, "frontend", Some("../frontend.git"));

    // Act
    git_vmr().current_dir(&root).arg("init").assert().success();

    // Assert
    assert_eq!(git_output(&root, ["log", "--format=%s"]), "initial");
    assert_eq!(
        read(&root.join(".gitignore")),
        "target\n/backend/\n/frontend/\n"
    );
}

#[test]
fn init_converts_a_legacy_vmr()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr")).expect("failed to create dir");
    fs::write(tmp.path().join(".gitvmr/config"), "[core]\nversion = 0\n")
        .expect("failed to write config");
    child_repo(tmp.path(), "backend", Some("../backend.git"));

    // Act
    git_vmr().current_dir(tmp.path()).arg("init").assert().success().stdout(
        predicate::str::starts_with(format!(
            "Removed legacy .gitvmr directory\n\
             Initialized virtual monorepo in {}\n\
             Declared 'backend'\n",
            shown_dir(tmp.path()).display()
        ))
    );

    // Assert
    assert!(read(&tmp.path().join(".gitvmr")).contains("[repo.backend]"));
    git_vmr().current_dir(tmp.path()).arg("status").assert().success();
}

#[test]
fn init_reports_reinitialized_without_changing_anything()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    child_repo(tmp.path(), "backend", Some("../backend.git"));
    git_vmr().current_dir(tmp.path()).arg("init").assert().success();
    let manifest = read(&tmp.path().join(".gitvmr"));
    child_repo(tmp.path(), "frontend", Some("../frontend.git"));

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(format!(
            "Reinitialized existing virtual monorepo in {}\n",
            shown_dir(tmp.path()).display()
        ))
        .stderr(predicate::str::is_empty());

    // Assert
    assert_eq!(read(&tmp.path().join(".gitvmr")), manifest);
    assert_eq!(read(&tmp.path().join(".gitignore")), "/backend/\n");
}

#[test]
fn init_with_working_dir_argument_initializes_requested_directory()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let target = tmp.path().join("target");
    fs::create_dir(&target).expect("failed to create target dir");

    // Act
    git_vmr()
        .arg("-C")
        .arg(&target)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::starts_with(format!(
            "Initialized virtual monorepo in {}\n",
            target.canonicalize().unwrap().display()
        )))
        .stderr(predicate::str::is_empty());

    // Assert
    assert!(target.join(".gitvmr").is_file());
    assert!(target.join(".git").is_dir());
    assert!(!tmp.path().join(".gitvmr").exists());
}

#[test]
fn init_with_existing_directory_argument_initializes_target()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let target = tmp.path().join("project");
    fs::create_dir(&target).expect("failed to create target dir");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .arg("project")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    // Assert
    assert!(target.join(".gitvmr").is_file());
    assert!(!tmp.path().join(".gitvmr").exists());
}

#[test]
fn init_with_missing_directory_argument_creates_target()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let target = tmp.path().join("project");

    // Act
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .arg("project")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    // Assert
    assert!(target.join(".gitvmr").is_file());
    assert!(target.join(".git").is_dir());
}

#[test]
fn init_directory_argument_is_relative_to_working_dir_argument()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let base = tmp.path().join("base");
    fs::create_dir(&base).expect("failed to create base dir");

    // Act
    git_vmr()
        .arg("-C")
        .arg(&base)
        .arg("init")
        .arg("project")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    // Assert
    assert!(base.join("project/.gitvmr").is_file());
    assert!(!tmp.path().join("project").exists());
}

#[test]
fn init_errors_when_directory_argument_is_file()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let file = tmp.path().join("file");
    fs::write(&file, "").expect("failed to write file");

    // Act + Assert
    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .arg("file")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal: cannot initialize")
                .and(predicate::str::contains("Not a directory"))
        );
}

#[test]
fn init_errors_when_working_dir_argument_does_not_exist()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let missing = tmp.path().join("missing");

    // Act
    git_vmr()
        .arg("-C")
        .arg(&missing)
        .arg("init")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal: cannot change to")
                .and(predicate::str::contains(missing.display().to_string()))
                .and(
                    predicate::str::contains("fatal: fatal: cannot change to")
                        .not()
                )
        );

    // Assert
    assert!(!tmp.path().join(".gitvmr").exists());
}

#[test]
fn version_flag_prints_package_version()
{
    // Act + Assert
    git_vmr()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "git-vmr {}",
            env!("CARGO_PKG_VERSION")
        )))
        .stderr(predicate::str::is_empty());
}
