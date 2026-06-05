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

fn commit_file_with_content(path: &Path, file: &str, content: &str)
{
    fs::write(path.join(file), content).expect("failed to write file");
    git(path, ["add", file]);
    git(path, ["commit", "-m", file]);
}

fn branch_exists(path: &Path, branch: &str) -> bool
{
    std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(path)
        .output()
        .expect("failed to run git")
        .status
        .success()
}

fn current_branch(path: &Path) -> String
{
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .expect("failed to run git");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn head_short(path: &Path) -> String
{
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .current_dir(path)
        .output()
        .expect("failed to run git");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn init_vmr_with_repos(tmp: &Path, repos: &[&str]) -> std::path::PathBuf
{
    let vmr = tmp.join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");

    for repo in repos
    {
        let repo_path = vmr.join(repo);
        init_repo(&repo_path);
        commit_file(&repo_path, "README.md");
    }

    vmr
}

fn add_worktrees(vmr: &Path)
{
    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();
}

#[test]
fn worktree_list_lists_main_aggregate_for_multiple_child_repositories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    let head = head_short(&vmr.join("backend"));

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!(
                "{} {} [master]",
                vmr.display(),
                head
            ))
            .and(predicate::str::contains("backend").not())
            .and(predicate::str::contains("frontend").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_lists_linked_aggregate_and_skips_non_git_children()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    add_worktrees(&vmr);
    let wt = tmp.path().join("wt");
    let head = head_short(&wt.join("backend"));

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("{} {} [wt]", wt.display(), head))
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_omits_unmarked_and_arbitrary_child_worktrees()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);

    git(&vmr.join("backend"), ["worktree", "add", "../../scratch/backend"]);
    git(&vmr.join("backend"), ["worktree", "add", "../../backend-only"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("scratch")
                .not()
                .and(predicate::str::contains("backend-only").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_reports_mixed_detached_partial_and_sorted_output()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["alpha", "tools", "zeta"]);
    fs::create_dir(tmp.path().join("alpha-wt"))
        .expect("failed to create alpha wt dir");
    fs::write(tmp.path().join("alpha-wt/.gitvmr"), "")
        .expect("failed to write marker");
    fs::create_dir(tmp.path().join("zeta-wt"))
        .expect("failed to create zeta wt dir");
    fs::write(tmp.path().join("zeta-wt/.gitvmr"), "")
        .expect("failed to write marker");

    git(&vmr.join("zeta"), ["worktree", "add", "../../zeta-wt/zeta"]);
    git(&vmr.join("alpha"), ["worktree", "add", "../../zeta-wt/alpha"]);
    git(&vmr.join("tools"), ["worktree", "add", "../../alpha-wt/tools"]);
    git(&tmp.path().join("zeta-wt/alpha"), ["checkout", "-b", "alpha-topic"]);
    git(&tmp.path().join("zeta-wt/zeta"), ["checkout", "--detach"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!(
                "{}",
                tmp.path().join("alpha-wt").display()
            ))
            .and(predicate::str::contains("[alpha-topic] (alpha)"))
            .and(predicate::str::contains("(detached HEAD) (zeta)"))
            .and(predicate::str::contains("[tools] (tools)"))
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_uses_child_working_dir_and_global_c()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);

    git_vmr()
        .current_dir(vmr.join("frontend"))
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}", vmr.display())))
        .stderr(predicate::str::is_empty());

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "-C",
            vmr.join("frontend").to_str().unwrap(),
            "worktree",
            "list"
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}", vmr.display())))
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_add_creates_child_worktrees_on_inferred_branch()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Preparing worktree (new branch 'wt') (backend, frontend)"
            )
            .or(predicate::str::contains(
                "Preparing worktree (new branch 'wt') (frontend, backend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&tmp.path().join("wt/backend")), "wt");
    assert_eq!(current_branch(&tmp.path().join("wt/frontend")), "wt");
}

#[test]
fn worktree_add_explicit_branch_creates_child_worktrees_on_requested_branch()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "-b", "feature/auth", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Preparing worktree (new branch 'feature/auth') (backend, frontend)"
            )
            .or(predicate::str::contains(
                "Preparing worktree (new branch 'feature/auth') (frontend, backend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&tmp.path().join("wt/backend")), "feature/auth");
    assert_eq!(current_branch(&tmp.path().join("wt/frontend")), "feature/auth");
    assert!(!branch_exists(&backend, "wt"));
    assert!(!branch_exists(&frontend, "wt"));
}

#[test]
fn worktree_add_explicit_branch_uses_commit_ish_as_start_point()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    git(&backend, ["branch", "main"]);
    git(&frontend, ["branch", "main"]);
    commit_file_with_content(&backend, "after-main.txt", "backend\n");
    commit_file_with_content(&frontend, "after-main.txt", "frontend\n");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "-b", "feature/auth", "../wt", "main"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Preparing worktree (new branch 'feature/auth') (backend, frontend)"
            )
            .or(predicate::str::contains(
                "Preparing worktree (new branch 'feature/auth') (frontend, backend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&tmp.path().join("wt/backend")), "feature/auth");
    assert_eq!(current_branch(&tmp.path().join("wt/frontend")), "feature/auth");
    assert!(!tmp.path().join("wt/backend/after-main.txt").exists());
    assert!(!tmp.path().join("wt/frontend/after-main.txt").exists());
}

#[test]
fn worktree_add_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    let backend = vmr.join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("backend")
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());

    assert!(tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/docs").exists());
}

#[test]
fn worktree_add_explicit_branch_creation_failures_are_reported_per_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    git(&backend, ["branch", "feature/auth"]);
    git(&frontend, ["branch", "feature/auth"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "-b", "feature/auth", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: a branch named 'feature/auth' already exists"
            )
            .and(predicate::str::contains("(backend, frontend)"))
        );

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
}

#[test]
fn worktree_add_explicit_checked_out_branch_fails_without_creating_inferred_branch()

{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "master"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: 'master' is already used by worktree"
            )
            .and(predicate::str::contains("(backend)"))
            .and(predicate::str::contains("(frontend)"))
        );

    assert!(!branch_exists(&backend, "wt"));
    assert!(!branch_exists(&frontend, "wt"));
}

#[test]
fn worktree_add_invalid_explicit_commit_ish_groups_failures()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "new"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "fatal: invalid reference: new (backend, frontend)"
        ));

    assert!(!branch_exists(&backend, "wt"));
    assert!(!branch_exists(&frontend, "wt"));
}

#[test]
fn worktree_add_best_effort_keeps_successful_child_worktrees()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    let tools = vmr.join("tools");
    init_repo(&backend);
    init_repo(&frontend);
    init_repo(&tools);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");
    commit_file(&tools, "README.md");
    git(&backend, ["branch", "wt"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("frontend")
                .and(predicate::str::contains("tools"))
                .and(predicate::str::contains("backend").not())
        )
        .stderr(predicate::str::contains(
            "fatal: a branch named 'wt' already exists (backend)"
        ));

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(tmp.path().join("wt/frontend").exists());
    assert!(tmp.path().join("wt/tools").exists());
}

#[test]
fn worktree_add_marker_enables_vmr_discovery_from_child_worktree()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    init_repo(&backend);
    commit_file(&backend, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();

    assert!(tmp.path().join("wt/.gitvmr").exists());
    git_vmr()
        .current_dir(tmp.path().join("wt/backend"))
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("On branch wt"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_add_uses_nested_working_dir_and_global_c_for_relative_target()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    init_repo(&backend);
    init_repo(&frontend);
    commit_file(&backend, "README.md");
    commit_file(&frontend, "README.md");

    git_vmr()
        .current_dir(&frontend)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();

    assert!(vmr.join("wt/backend").exists());
    assert!(vmr.join("wt/frontend").exists());

    git_vmr()
        .arg("-C")
        .arg(&frontend)
        .args(["worktree", "add", "../wt-c"])
        .assert()
        .success();

    assert!(vmr.join("wt-c/backend").exists());
    assert!(vmr.join("wt-c/frontend").exists());
}

#[test]
fn worktree_add_reports_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    fs::create_dir(vmr.join(".gitvmr")).expect("failed to create marker");
    let alpha = vmr.join("alpha");
    let zeta = vmr.join("zeta");
    init_repo(&alpha);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&zeta, "README.md");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "new"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with(
            "fatal: invalid reference: new (alpha, zeta)\n"
        ));
}

#[test]
fn worktree_remove_removes_child_worktrees()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    add_worktrees(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
}

#[test]
fn worktree_remove_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    add_worktrees(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/docs").exists());
}

#[test]
fn worktree_remove_dirty_child_fails_without_force_and_reports_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    fs::write(tmp.path().join("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal:")
                .and(predicate::str::contains("(backend)"))
        );

    assert!(tmp.path().join("wt/backend").exists());
}

#[test]
fn worktree_remove_force_removes_dirty_child()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    fs::write(tmp.path().join("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "--force", "../wt"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/backend").exists());
}

#[test]
fn worktree_remove_double_force_removes_locked_child()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "--force", "--force", "../wt"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/backend").exists());
}

#[test]
fn worktree_remove_repeated_short_force_removes_locked_child()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "-ff", "../wt"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/backend").exists());
}

#[test]
fn worktree_remove_best_effort_keeps_successful_removals()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr =
        init_vmr_with_repos(tmp.path(), &["backend", "frontend", "tools"]);
    add_worktrees(&vmr);
    fs::write(tmp.path().join("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));

    assert!(tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
    assert!(!tmp.path().join("wt/tools").exists());
}

#[test]
fn worktree_remove_success_cleans_marker_and_empty_aggregate_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    add_worktrees(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/.gitvmr").exists());
    assert!(!tmp.path().join("wt").exists());
}

#[test]
fn worktree_remove_partial_failure_keeps_marker()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    add_worktrees(&vmr);
    fs::write(tmp.path().join("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .failure();

    assert!(tmp.path().join("wt/.gitvmr").exists());
    assert!(tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
}

#[test]
fn worktree_remove_uses_nested_working_dir_and_global_c_for_relative_target()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);

    git_vmr()
        .current_dir(vmr.join("frontend"))
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();
    git_vmr()
        .current_dir(vmr.join("frontend"))
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success();
    assert!(!vmr.join("wt/backend").exists());
    assert!(!vmr.join("wt/frontend").exists());

    git_vmr()
        .arg("-C")
        .arg(vmr.join("frontend"))
        .args(["worktree", "add", "../wt-c"])
        .assert()
        .success();
    git_vmr()
        .arg("-C")
        .arg(vmr.join("frontend"))
        .args(["worktree", "remove", "../wt-c"])
        .assert()
        .success();
    assert!(!vmr.join("wt-c/backend").exists());
    assert!(!vmr.join("wt-c/frontend").exists());
}

#[test]
fn worktree_remove_reports_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["alpha", "zeta"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "remove", "../missing"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("(alpha)")
                .and(predicate::str::contains("(zeta)"))
                .and(predicate::str::starts_with("fatal:"))
        );
}

#[test]
fn worktree_move_moves_child_worktrees()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    add_worktrees(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
    assert!(tmp.path().join("moved/backend").exists());
    assert!(tmp.path().join("moved/frontend").exists());
}

#[test]
fn worktree_move_skips_non_git_child_directories()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    add_worktrees(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(tmp.path().join("moved/backend").exists());
    assert!(!tmp.path().join("moved/docs").exists());
}

#[test]
fn worktree_move_locked_child_fails_without_force_and_reports_repository()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal:")
                .and(predicate::str::contains("(backend)"))
        );

    assert!(tmp.path().join("wt/backend").exists());
}

#[test]
fn worktree_move_dirty_child_is_delegated_to_git()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    fs::write(tmp.path().join("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(tmp.path().join("moved/backend").exists());
}

#[test]
fn worktree_move_force_moves_dirty_child()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);
    add_worktrees(&vmr);
    fs::write(tmp.path().join("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "--force", "../wt", "../moved"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(tmp.path().join("moved/backend").exists());
}

#[test]
fn worktree_move_best_effort_keeps_successful_moves()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr =
        init_vmr_with_repos(tmp.path(), &["backend", "frontend", "tools"]);
    add_worktrees(&vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));

    assert!(tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
    assert!(!tmp.path().join("wt/tools").exists());
    assert!(tmp.path().join("moved/frontend").exists());
    assert!(tmp.path().join("moved/tools").exists());
}

#[test]
fn worktree_move_success_cleans_source_marker_and_empty_aggregate_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    add_worktrees(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success();

    assert!(!tmp.path().join("wt/.gitvmr").exists());
    assert!(!tmp.path().join("wt").exists());
    assert!(tmp.path().join("moved/.gitvmr").exists());
}

#[test]
fn worktree_move_partial_failure_keeps_source_and_destination_markers()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);
    add_worktrees(&vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure();

    assert!(tmp.path().join("wt/.gitvmr").exists());
    assert!(tmp.path().join("moved/.gitvmr").exists());
    assert!(tmp.path().join("wt/backend").exists());
    assert!(tmp.path().join("moved/frontend").exists());
}

#[test]
fn worktree_move_total_failure_leaves_destination_marker()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure();

    assert!(tmp.path().join("moved/.gitvmr").exists());
}

#[test]
fn worktree_move_uses_nested_working_dir_and_global_c_for_relative_paths()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["backend", "frontend"]);

    git_vmr()
        .current_dir(vmr.join("frontend"))
        .args(["worktree", "add", "../wt"])
        .assert()
        .success();
    git_vmr()
        .current_dir(vmr.join("frontend"))
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success();
    assert!(vmr.join("moved/backend").exists());
    assert!(vmr.join("moved/frontend").exists());

    git_vmr()
        .arg("-C")
        .arg(vmr.join("frontend"))
        .args(["worktree", "move", "../moved", "../moved-c"])
        .assert()
        .success();
    assert!(vmr.join("moved-c/backend").exists());
    assert!(vmr.join("moved-c/frontend").exists());
}

#[test]
fn worktree_move_reports_failures_in_repository_name_order()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = init_vmr_with_repos(tmp.path(), &["alpha", "zeta"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "move", "../missing", "../moved"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("(alpha)")
                .and(predicate::str::contains("(zeta)"))
                .and(predicate::str::starts_with("fatal:"))
        );
}
