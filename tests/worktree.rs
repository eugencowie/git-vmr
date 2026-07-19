mod common;

use common::{
    TestVmr, commit_file, git, git_command, git_vmr, init_repo, write_commit
};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn branch_exists(path: &Path, branch: &str) -> bool
{
    git_command()
        .args(["rev-parse", "--verify", branch])
        .current_dir(path)
        .output()
        .expect("failed to run git")
        .status
        .success()
}

fn current_branch(path: &Path) -> String
{
    let output = git_command()
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .expect("failed to run git");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn head_short(path: &Path) -> String
{
    let output = git_command()
        .args(["rev-parse", "--short=8", "HEAD"])
        .current_dir(path)
        .output()
        .expect("failed to run git");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn count_occurrences(haystack: &str, needle: &str) -> usize
{
    haystack.match_indices(needle).count()
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
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("{} [master]", vmr.display()))
                .and(predicate::str::contains("backend").not())
                .and(predicate::str::contains("frontend").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_without_subcommand_matches_worktree_list()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();

    let list = git_vmr()
        .current_dir(vmr)
        .args(["worktree", "list"])
        .output()
        .expect("failed to run git-vmr worktree list");
    let bare = git_vmr()
        .current_dir(vmr)
        .args(["worktree"])
        .output()
        .expect("failed to run git-vmr worktree");

    assert_eq!(bare.status, list.status);
    assert_eq!(bare.stdout, list.stdout);
    assert_eq!(bare.stderr, list.stderr);
}

#[test]
fn worktree_list_lists_linked_aggregate_and_skips_non_git_children()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    add_worktrees(vmr);
    let wt = fixture.sibling("wt");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("{} [wt]", wt.display()))
                .and(predicate::str::contains("docs").not())
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_reports_shared_branch_with_different_child_heads_once()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    let wt = fixture.sibling("wt");

    write_commit(
        &wt.join("backend"),
        "backend.txt",
        "backend\n",
        "backend.txt"
    );
    let backend_head = head_short(&wt.join("backend"));
    let frontend_head = head_short(&wt.join("frontend"));
    assert_ne!(backend_head, frontend_head);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(predicate::function(move |stdout: &str| {
            count_occurrences(stdout, &format!("{} [wt]", wt.display())) == 1
                && !stdout.contains(&backend_head)
                && !stdout.contains(&frontend_head)
        }))
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_omits_unmarked_and_arbitrary_child_worktrees()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();

    git(&vmr.join("backend"), ["worktree", "add", "../../scratch/backend"]);
    git(&vmr.join("backend"), ["worktree", "add", "../../backend-only"]);

    git_vmr()
        .current_dir(vmr)
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
    let fixture = TestVmr::with_repos(&["alpha", "tools", "zeta"]);
    let vmr = fixture.path();
    fs::create_dir(fixture.sibling("alpha-wt"))
        .expect("failed to create alpha wt dir");
    fs::write(fixture.sibling("alpha-wt/.gitvmr"), "")
        .expect("failed to write marker");
    fs::create_dir(fixture.sibling("zeta-wt"))
        .expect("failed to create zeta wt dir");
    fs::write(fixture.sibling("zeta-wt/.gitvmr"), "")
        .expect("failed to write marker");

    git(&vmr.join("zeta"), ["worktree", "add", "../../zeta-wt/zeta"]);
    git(&vmr.join("alpha"), ["worktree", "add", "../../zeta-wt/alpha"]);
    git(&vmr.join("tools"), ["worktree", "add", "../../alpha-wt/tools"]);
    git(&fixture.sibling("zeta-wt/alpha"), ["checkout", "-b", "alpha-topic"]);
    git(&fixture.sibling("zeta-wt/zeta"), ["checkout", "--detach"]);
    let zeta_head = head_short(&fixture.sibling("zeta-wt/zeta"));

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(predicate::function(move |stdout: &str| {
            let alpha_wt = format!("{}", fixture.sibling("alpha-wt").display());
            let zeta_wt = format!("{}", fixture.sibling("zeta-wt").display());

            count_occurrences(stdout, &alpha_wt) == 1
                && count_occurrences(stdout, &zeta_wt) == 1
                && stdout.contains(&format!("{alpha_wt} [tools] (tools)"))
                && stdout.contains("  [alpha-topic] (alpha)")
                && stdout
                    .contains(&format!("  {zeta_head} (detached HEAD) (zeta)"))
        }))
        .stderr(predicate::str::is_empty());
}

#[test]
fn worktree_list_uses_child_working_dir_and_global_c()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr.join("frontend"))
        .args(["worktree", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}", vmr.display())))
        .stderr(predicate::str::is_empty());

    git_vmr()
        .current_dir(vmr.parent().unwrap())
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
            predicate::str::contains("Preparing worktree (new branch 'wt')")
                .and(predicate::str::contains("(backend, frontend)").not())
                .and(predicate::str::contains("(backend)").not())
                .and(predicate::str::contains("(frontend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&tmp.path().join("wt/backend")), "wt");
    assert_eq!(current_branch(&tmp.path().join("wt/frontend")), "wt");
}

#[test]
fn worktree_add_checks_out_existing_local_inferred_branch()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    git(&backend, ["branch", "wt"]);
    git(&frontend, ["branch", "wt"]);
    write_commit(&backend, "after-wt.txt", "backend\n", "after-wt.txt");
    write_commit(&frontend, "after-wt.txt", "frontend\n", "after-wt.txt");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Preparing worktree (checking out 'wt')")
                .and(predicate::str::contains("(backend, frontend)").not())
                .and(predicate::str::contains("(backend)").not())
                .and(predicate::str::contains("(frontend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&fixture.sibling("wt/backend")), "wt");
    assert_eq!(current_branch(&fixture.sibling("wt/frontend")), "wt");
    assert!(!fixture.sibling("wt/backend/after-wt.txt").exists());
    assert!(!fixture.sibling("wt/frontend/after-wt.txt").exists());
}

#[test]
fn worktree_add_mixes_existing_and_created_inferred_branch()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    let backend = vmr.join("backend");
    let frontend = vmr.join("frontend");
    git(&backend, ["branch", "wt"]);
    write_commit(&backend, "after-wt.txt", "backend\n", "after-wt.txt");
    write_commit(&frontend, "after-wt.txt", "frontend\n", "after-wt.txt");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Preparing worktree (checking out 'wt') (backend)"
            )
            .and(predicate::str::contains(
                "Preparing worktree (new branch 'wt') (frontend)"
            ))
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&fixture.sibling("wt/backend")), "wt");
    assert_eq!(current_branch(&fixture.sibling("wt/frontend")), "wt");
    assert!(!fixture.sibling("wt/backend/after-wt.txt").exists());
    assert!(fixture.sibling("wt/frontend/after-wt.txt").exists());
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
                "Preparing worktree (new branch 'feature/auth')"
            )
            .and(predicate::str::contains("(backend, frontend)").not())
            .and(predicate::str::contains("(backend)").not())
            .and(predicate::str::contains("(frontend)").not())
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
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    let backend = fixture.repo("backend");
    let frontend = fixture.repo("frontend");
    git(&backend, ["branch", "main"]);
    git(&frontend, ["branch", "main"]);
    write_commit(&backend, "after-main.txt", "backend\n", "after-main.txt");
    write_commit(&frontend, "after-main.txt", "frontend\n", "after-main.txt");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "-b", "feature/auth", "../wt", "main"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "Preparing worktree (new branch 'feature/auth')"
            )
            .and(predicate::str::contains("(backend, frontend)").not())
            .and(predicate::str::contains("(backend)").not())
            .and(predicate::str::contains("(frontend)").not())
        )
        .stderr(predicate::str::is_empty());

    assert_eq!(current_branch(&fixture.sibling("wt/backend")), "feature/auth");
    assert_eq!(current_branch(&fixture.sibling("wt/frontend")), "feature/auth");
    assert!(!fixture.sibling("wt/backend/after-main.txt").exists());
    assert!(!fixture.sibling("wt/frontend/after-main.txt").exists());
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
            predicate::str::contains("Preparing worktree (new branch 'wt')")
                .and(predicate::str::contains("(backend)").not())
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
            .and(predicate::str::contains("(backend, frontend)").not())
            .and(predicate::str::contains("(backend)").not())
            .and(predicate::str::contains("(frontend)").not())
        );

    assert!(!tmp.path().join("wt/backend").exists());
    assert!(!tmp.path().join("wt/frontend").exists());
}

#[test]
fn worktree_add_existing_inferred_branch_checked_out_elsewhere_fails()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    let backend = vmr.join("backend");
    git(&backend, ["branch", "wt"]);
    let occupied = fixture.sibling("occupied-backend");
    git(&backend, ["worktree", "add", occupied.to_str().unwrap(), "wt"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal: 'wt' is already used by worktree")
                .and(predicate::str::contains("(backend)").not())
        );

    assert!(!fixture.sibling("wt/backend").exists());
}

#[test]
fn worktree_add_bad_inferred_branch_ref_does_not_skip_remaining_repos()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    fs::write(vmr.join("backend/.git/refs/heads/wt"), "not-a-sha\n")
        .expect("failed to corrupt branch ref");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("frontend"))
        .stderr(
            predicate::str::contains("failed to check branch 'wt'")
                .and(predicate::str::contains("backend"))
        );

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(fixture.sibling("wt/frontend").exists());
    assert_eq!(current_branch(&fixture.sibling("wt/frontend")), "wt");
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
        .stderr(
            predicate::str::contains("fatal: invalid reference: new")
                .and(predicate::str::contains("(backend, frontend)").not())
                .and(predicate::str::contains("(backend)").not())
                .and(predicate::str::contains("(frontend)").not())
        );

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
    let occupied = tmp.path().join("occupied-backend");
    git(&backend, ["worktree", "add", occupied.to_str().unwrap(), "wt"]);

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
            "fatal: 'wt' is already used by worktree"
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
    let has_ref = vmr.join("has-ref");
    let zeta = vmr.join("zeta");
    init_repo(&alpha);
    init_repo(&has_ref);
    init_repo(&zeta);
    commit_file(&alpha, "README.md");
    commit_file(&has_ref, "README.md");
    commit_file(&zeta, "README.md");
    git(&has_ref, ["branch", "new"]);

    git_vmr()
        .current_dir(&vmr)
        .args(["worktree", "add", "../wt", "new"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with(
            "fatal: invalid reference: new (alpha, zeta)\n"
        ));
}

#[test]
fn worktree_remove_removes_child_worktrees()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
}

#[test]
fn worktree_remove_rm_alias_removes_child_worktrees()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "rm", "../wt"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
}

#[test]
fn worktree_remove_skips_non_git_child_directories()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/docs").exists());
}

#[test]
fn worktree_remove_dirty_child_fails_without_force_and_reports_repository()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal:")
                .and(predicate::str::contains("(backend)").not())
        );

    assert!(fixture.sibling("wt/backend").exists());
}

#[test]
fn worktree_remove_force_removes_dirty_child()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--force", "../wt"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/backend").exists());
}

#[test]
fn worktree_remove_double_force_removes_locked_child()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--force", "--force", "../wt"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/backend").exists());
}

#[test]
fn worktree_remove_repeated_short_force_removes_locked_child()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "-ff", "../wt"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/backend").exists());
}

#[test]
fn worktree_remove_best_effort_keeps_successful_removals()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend", "tools"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));

    assert!(fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
    assert!(!fixture.sibling("wt/tools").exists());
}

#[test]
fn worktree_remove_success_cleans_marker_and_empty_aggregate_directory()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/.gitvmr").exists());
    assert!(!fixture.sibling("wt").exists());
}

#[test]
fn worktree_remove_delete_removes_child_worktrees_and_branches()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--delete", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch wt"))
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
    assert!(!branch_exists(&vmr.join("backend"), "wt"));
    assert!(!branch_exists(&vmr.join("frontend"), "wt"));
}

#[test]
fn worktree_remove_delete_uses_actual_child_worktree_branch()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "-b", "feature/auth", "../wt"])
        .assert()
        .success();

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--delete", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch feature/auth"))
        .stderr(predicate::str::is_empty());

    assert!(!branch_exists(&vmr.join("backend"), "feature/auth"));
    assert!(!branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_force_delete_deletes_unmerged_branch()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    write_commit(
        &fixture.sibling("wt/backend"),
        "topic.txt",
        "topic\n",
        "topic.txt"
    );

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "-D", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch wt"))
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_force_and_force_delete_removes_dirty_worktree_and_unmerged_branch()

{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    write_commit(
        &fixture.sibling("wt/backend"),
        "topic.txt",
        "topic\n",
        "topic.txt"
    );
    fs::write(fixture.sibling("wt/backend/dirty.txt"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--force", "-D", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch wt"))
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_force_delete_uses_actual_child_worktree_branch()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "add", "-b", "feature/auth", "../wt"])
        .assert()
        .success();
    write_commit(
        &fixture.sibling("wt/backend"),
        "topic.txt",
        "topic\n",
        "topic.txt"
    );

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "-D", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch feature/auth"))
        .stderr(predicate::str::is_empty());

    assert!(!branch_exists(&vmr.join("backend"), "feature/auth"));
    assert!(!branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_delete_skips_detached_child_worktree()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&fixture.sibling("wt/backend"), ["checkout", "--detach"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--delete", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch").not())
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_force_delete_skips_detached_child_worktree()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&fixture.sibling("wt/backend"), ["checkout", "--detach"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "-D", "../wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch").not())
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_delete_skips_branch_for_failed_child_removal()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--delete", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Deleted branch wt (frontend)"))
        .stderr(predicate::str::contains("(backend)"));

    assert!(fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
    assert!(branch_exists(&vmr.join("backend"), "wt"));
    assert!(!branch_exists(&vmr.join("frontend"), "wt"));
}

#[test]
fn worktree_remove_force_delete_skips_branch_for_failed_child_removal()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "-D", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Deleted branch wt (frontend)"))
        .stderr(predicate::str::contains("(backend)"));

    assert!(fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
    assert!(branch_exists(&vmr.join("backend"), "wt"));
    assert!(!branch_exists(&vmr.join("frontend"), "wt"));
}

#[test]
fn worktree_remove_delete_failure_cleans_worktree_root()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    write_commit(
        &fixture.sibling("wt/backend"),
        "topic.txt",
        "topic\n",
        "topic.txt"
    );

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--delete", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            // The failure covers every repo in scope: no suffix
            predicate::str::contains("branch 'wt'")
                .and(predicate::str::contains("(backend)").not())
        );

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/.gitvmr").exists());
    assert!(!fixture.sibling("wt").exists());
    assert!(branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_force_delete_does_not_force_delete_unmerged_branch()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    write_commit(
        &fixture.sibling("wt/backend"),
        "README.md",
        "dirty\n",
        "topic"
    );

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--force", "--delete", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            // The failure covers every repo in scope: no suffix
            predicate::str::contains("branch 'wt'")
                .and(predicate::str::contains("(backend)").not())
        );

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(branch_exists(&vmr.join("backend"), "wt"));
}

#[test]
fn worktree_remove_delete_reports_branch_failures_in_repository_name_order()
{
    // The middle repo stays merged, so the failure group does not cover
    // the whole scope and the suffix names the failing repos.
    let fixture = TestVmr::with_repos(&["alpha", "mid", "zeta"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    write_commit(
        &fixture.sibling("wt/alpha"),
        "topic.txt",
        "alpha\n",
        "topic.txt"
    );
    write_commit(
        &fixture.sibling("wt/zeta"),
        "topic.txt",
        "zeta\n",
        "topic.txt"
    );

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "--delete", "../wt"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Deleted branch wt (mid)"))
        .stderr(predicate::str::contains("(alpha, zeta)"));
}

#[test]
fn worktree_remove_partial_failure_keeps_marker()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "remove", "../wt"])
        .assert()
        .failure();

    assert!(fixture.sibling("wt/.gitvmr").exists());
    assert!(fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
}

#[test]
fn worktree_remove_uses_nested_working_dir_and_global_c_for_relative_target()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();

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
    let fixture = TestVmr::with_repos(&["alpha", "zeta"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr)
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
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
    assert!(fixture.sibling("moved/backend").exists());
    assert!(fixture.sibling("moved/frontend").exists());
}

#[test]
fn worktree_move_skips_non_git_child_directories()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    fs::create_dir(vmr.join("docs")).expect("failed to create docs dir");
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(fixture.sibling("moved/backend").exists());
    assert!(!fixture.sibling("moved/docs").exists());
}

#[test]
fn worktree_move_locked_child_fails_without_force_and_reports_repository()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("fatal:")
                .and(predicate::str::contains("(backend)").not())
        );

    assert!(fixture.sibling("wt/backend").exists());
}

#[test]
fn worktree_move_dirty_child_is_delegated_to_git()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(fixture.sibling("moved/backend").exists());
}

#[test]
fn worktree_move_force_moves_dirty_child()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    fs::write(fixture.sibling("wt/backend/README.md"), "dirty\n")
        .expect("failed to dirty worktree");

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "--force", "../wt", "../moved"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/backend").exists());
    assert!(fixture.sibling("moved/backend").exists());
}

#[test]
fn worktree_move_best_effort_keeps_successful_moves()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend", "tools"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("(backend)"));

    assert!(fixture.sibling("wt/backend").exists());
    assert!(!fixture.sibling("wt/frontend").exists());
    assert!(!fixture.sibling("wt/tools").exists());
    assert!(fixture.sibling("moved/frontend").exists());
    assert!(fixture.sibling("moved/tools").exists());
}

#[test]
fn worktree_move_success_cleans_source_marker_and_empty_aggregate_directory()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .success();

    assert!(!fixture.sibling("wt/.gitvmr").exists());
    assert!(!fixture.sibling("wt").exists());
    assert!(fixture.sibling("moved/.gitvmr").exists());
}

#[test]
fn worktree_move_partial_failure_keeps_source_and_destination_markers()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();
    add_worktrees(vmr);
    git(&vmr.join("backend"), ["worktree", "lock", "../../wt/backend"]);

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure();

    assert!(fixture.sibling("wt/.gitvmr").exists());
    assert!(fixture.sibling("moved/.gitvmr").exists());
    assert!(fixture.sibling("wt/backend").exists());
    assert!(fixture.sibling("moved/frontend").exists());
}

#[test]
fn worktree_move_total_failure_leaves_destination_marker()
{
    let fixture = TestVmr::with_repos(&["backend"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr)
        .args(["worktree", "move", "../wt", "../moved"])
        .assert()
        .failure();

    assert!(fixture.sibling("moved/.gitvmr").exists());
}

#[test]
fn worktree_move_uses_nested_working_dir_and_global_c_for_relative_paths()
{
    let fixture = TestVmr::with_repos(&["backend", "frontend"]);
    let vmr = fixture.path();

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
    let fixture = TestVmr::with_repos(&["alpha", "zeta"]);
    let vmr = fixture.path();

    git_vmr()
        .current_dir(vmr)
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
