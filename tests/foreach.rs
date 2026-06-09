mod common;

use common::git_vmr;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn create_vmr() -> tempfile::TempDir
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    fs::create_dir(tmp.path().join(".gitvmr"))
        .expect("failed to create marker");
    tmp
}

fn create_child_repo(vmr: &Path, name: &str)
{
    let repo = vmr.join(name);
    fs::create_dir(&repo).expect("failed to create child repo");
    fs::create_dir(repo.join(".git")).expect("failed to create git marker");
}

#[test]
fn foreach_runs_command_across_child_repos_and_skips_non_git_children()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non-git child");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "printf '%s\\n' \"$name\""])
        .assert()
        .success()
        .stdout("backend\nfrontend\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_succeeds_without_output_when_no_child_repos_exist()
{
    let tmp = create_vmr();
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non-git child");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "pwd"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_errors_outside_a_vmr()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "pwd"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "fatal: not a virtual monorepo (or any of the parent directories): .gitvmr"
        ));
}

#[test]
fn foreach_renders_parallel_results_in_repository_order()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "alpha");
    create_child_repo(tmp.path(), "zeta");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            "if [ \"$name\" = zeta ]; then sleep 0.2; fi; echo \"$name\""
        ])
        .assert()
        .success()
        .stdout("alpha\nzeta\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_replays_stdout_stderr_and_default_headers()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "echo out; echo err >&2"])
        .assert()
        .success()
        .stdout("Entering 'backend'\nout\n")
        .stderr("err\n");
}

#[test]
fn foreach_quiet_suppresses_headers_only()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "echo ok"])
        .assert()
        .success()
        .stdout("ok\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_reports_single_child_failure()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "false"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains(
                "fatal: command failed in 'backend' with exit status 1"
            )
            .and(predicate::str::contains("fatal: foreach failed"))
        );
}

#[test]
fn foreach_reports_multiple_failures_in_repository_order()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "alpha");
    create_child_repo(tmp.path(), "zeta");

    let output = git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "false"])
        .output()
        .expect("failed to run git-vmr");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let alpha = stderr
        .find("fatal: command failed in 'alpha' with exit status 1")
        .expect("missing alpha failure");
    let zeta = stderr
        .find("fatal: command failed in 'zeta' with exit status 1")
        .expect("missing zeta failure");
    assert!(alpha < zeta, "{stderr}");
}

#[test]
fn foreach_does_not_report_successful_repositories_as_failures()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "test \"$name\" = frontend"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("backend")
                .and(predicate::str::contains("frontend").not())
        );
}

#[test]
fn foreach_sets_vmr_environment_variables_without_sha1()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            "printf '%s|%s|%s|%s|%s\\n' \"$name\" \"$sm_path\" \"$displaypath\" \"$toplevel\" \"${sha1-unset}\""
        ])
        .assert()
        .success()
        .stdout(format!(
            "backend|backend|backend|{}|unset\n",
            tmp.path().canonicalize().unwrap().display()
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_displaypath_respects_effective_working_directory()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");

    git_vmr()
        .current_dir(tmp.path().join("frontend"))
        .args(["foreach", "--quiet", "echo \"$name:$displaypath\""])
        .assert()
        .success()
        .stdout("backend:../backend\nfrontend:.\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_closes_child_stdin()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "cat"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_discovers_vmr_from_nested_working_directory_and_global_c()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");
    fs::create_dir_all(tmp.path().join("frontend/src"))
        .expect("failed to create nested dir");

    git_vmr()
        .arg("-C")
        .arg(tmp.path().join("frontend/src"))
        .args(["foreach", "--quiet", "printf '%s\\n' \"$name\""])
        .assert()
        .success()
        .stdout("backend\nfrontend\n")
        .stderr(predicate::str::is_empty());
}
