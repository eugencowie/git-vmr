mod common;

use common::{commit_file, git, git_vmr, init_repo, init_vmr, shown_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn status_errors_outside_a_vmr()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: not a virtual monorepo (or any of the parent directories): .gitvmr"
            )
            .and(predicate::str::contains("fatal: fatal: not a virtual monorepo").not())
        );
}

#[test]
fn status_errors_in_a_legacy_vmr()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr")).expect("failed to create dir");

    // Act + Assert
    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(format!(
            "fatal: '{}' is a legacy virtual monorepo\n\
             hint: run 'git vmr init' there to convert it\n",
            shown_dir(tmp.path()).display()
        ));
}

#[test]
fn status_errors_on_an_invalid_manifest()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &[]);
    fs::write(tmp.path().join(".gitvmr"), "[repo.backend]\nulr = \"u\"\n")
        .expect("failed to write manifest");

    // Act + Assert
    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::starts_with("fatal: invalid manifest")
                .and(predicate::str::contains("unknown field `ulr`"))
        );
}

#[test]
fn status_warns_about_declared_repos_that_are_not_cloned()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["api", "backend"]);
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");

    // Act + Assert: status warns
    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("On branch master"))
        .stderr(
            "warning: 'api' declared in .gitvmr but not cloned; run 'git vmr \
             clone' to clone them\n"
        );

    // Act + Assert: routing into the missing repo fails
    git_vmr()
        .current_dir(tmp.path())
        .args(["add", "api/file.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "'api' is not cloned; run 'git vmr clone' to clone it"
        ));
}

#[test]
fn status_warns_about_declared_repos_that_are_not_ignored()
{
    // Arrange
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend", "frontend"]);
    fs::write(tmp.path().join(".gitignore"), "/fr*/\n")
        .expect("failed to write .gitignore");
    for name in ["backend", "frontend"]
    {
        let repo = tmp.path().join(name);
        init_repo(&repo);
        commit_file(&repo, "README.md");
    }

    // Act + Assert
    git_vmr().current_dir(tmp.path()).arg("status").assert().success().stderr(
        "warning: 'backend' is not ignored by the root repo; add \
             '/backend/' to .gitignore\n"
    );
}

#[test]
fn status_skips_non_git_dirs_alongside_child_repos()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend"]);
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non repo dir");
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("On branch master")
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_in_vmr_root_reports_child_repo_changes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend"]);
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");
    fs::write(repo.join("src.rs"), "changed\n").expect("failed to write file");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("On branch master")
                .and(predicate::str::contains("backend/src.rs"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_reports_clean_summary_for_clean_child_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend"]);
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout("On branch master\n\nnothing to commit, working tree clean\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_keeps_initial_repo_separate_from_committed_repo_on_same_branch()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["committed", "new-repo"]);
    let committed = tmp.path().join("committed");
    let new_repo = tmp.path().join("new-repo");
    init_repo(&committed);
    init_repo(&new_repo);
    commit_file(&committed, "README.md");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "On branch master (., committed)\n\n\
                 nothing to commit, working tree clean\n\n"
            )
            .and(predicate::str::contains(
                "On branch master (new-repo)\n\nNo commits yet\n"
            ))
            .and(
                predicate::str::contains(
                    "On branch master (committed, new-repo)"
                )
                .not()
            )
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_uses_nested_working_dir_for_discovery_and_relative_paths()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend", "frontend"]);
    let backend = tmp.path().join("backend");
    let frontend = tmp.path().join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    fs::write(backend.join("src.rs"), "changed\n")
        .expect("failed to write file");

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("../backend/src.rs"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_with_no_child_repos_reports_only_the_root_repo()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &[]);
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non repo dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout("On branch master\n\nnothing to commit, working tree clean\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_reports_root_repo_changes_relative_to_the_working_dir()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend"]);
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");
    fs::write(tmp.path().join(".gitignore"), "/backend/\n/target/\n")
        .expect("failed to write file");
    fs::create_dir(tmp.path().join("docs")).expect("failed to create dir");
    fs::write(tmp.path().join("docs/new.md"), "content\n")
        .expect("failed to write file");

    git_vmr()
        .current_dir(&repo)
        .arg("status")
        .assert()
        .success()
        .stdout(
            "On branch master\n\
             \n\
             Changes not staged for commit:\n  \
             (use \"git vmr add <file>...\" to update what will be committed)\n  \
             (use \"git vmr restore <file>...\" to discard changes in working directory)\n\
             \tmodified:   ../.gitignore\n\
             \n\
             Untracked files:\n  \
             (use \"git vmr add <file>...\" to include in what will be committed)\n\
             \tnew file:   ../docs/\n"
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_reports_staged_changes()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["backend"]);
    let repo = tmp.path().join("backend");
    init_repo(&repo);
    commit_file(&repo, "README.md");
    fs::write(repo.join("new.rs"), "content\n").expect("failed to write file");
    git(&repo, ["add", "new.rs"]);

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Changes to be committed:")
                .and(predicate::str::contains("backend/new.rs"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn status_fails_on_corrupted_git_dir()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), &["broken"]);
    let repo = tmp.path().join("broken");
    fs::create_dir(&repo).expect("failed to create repo");
    fs::write(repo.join(".git"), "not a gitfile\n")
        .expect("failed to corrupt git dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to read git status"));
}
