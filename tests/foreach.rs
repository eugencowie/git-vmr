mod common;

use common::{git_vmr, init_vmr, shown_dir};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[cfg(unix)]
const CHILD_NEWLINE: &str = "\n";
#[cfg(windows)]
const CHILD_NEWLINE: &str = "\r\n";

#[cfg(unix)]
fn shell_command<'a>(unix: &'a str, _windows: &'a str) -> &'a str
{
    unix
}

#[cfg(windows)]
fn shell_command<'a>(_unix: &'a str, windows: &'a str) -> &'a str
{
    windows
}

fn child_lines(lines: &[&str]) -> String
{
    format!("{}{}", lines.join(CHILD_NEWLINE), CHILD_NEWLINE)
}

fn create_vmr(repos: &[&str]) -> tempfile::TempDir
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    init_vmr(tmp.path(), repos);
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
    let tmp = create_vmr(&["backend", "frontend"]);
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non-git child");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            shell_command("printf '%s\\n' \"$name\"", "echo %name%")
        ])
        .assert()
        .success()
        .stdout(child_lines(&[".", "backend", "frontend"]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_runs_only_in_the_root_repo_when_no_child_repos_exist()
{
    let tmp = create_vmr(&[]);
    fs::create_dir(tmp.path().join("docs"))
        .expect("failed to create non-git child");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "echo ok"])
        .assert()
        .success()
        .stdout(format!("Entering '.'\nok{CHILD_NEWLINE}"))
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
    let tmp = create_vmr(&["alpha", "zeta"]);
    create_child_repo(tmp.path(), "alpha");
    create_child_repo(tmp.path(), "zeta");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            shell_command(
                "if [ \"$name\" = zeta ]; then sleep 0.2; fi; echo \"$name\"",
                "if %name%==zeta (ping -n 2 127.0.0.1 >NUL&echo %name%) else echo %name%"
            )
        ])
        .assert()
        .success()
        .stdout(child_lines(&[".", "alpha", "zeta"]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_replays_stdout_stderr_and_default_headers()
{
    let tmp = create_vmr(&["backend"]);
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            shell_command("echo out; echo err >&2", "echo out&echo err>&2")
        ])
        .assert()
        .success()
        .stdout(format!(
            "Entering '.'\nout{CHILD_NEWLINE}\
             Entering 'backend'\nout{CHILD_NEWLINE}"
        ))
        .stderr(format!("err{CHILD_NEWLINE}err{CHILD_NEWLINE}"));
}

#[test]
fn foreach_quiet_suppresses_headers_only()
{
    let tmp = create_vmr(&["backend"]);
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", "echo ok"])
        .assert()
        .success()
        .stdout(child_lines(&["ok", "ok"]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_reports_single_child_failure()
{
    let tmp = create_vmr(&["backend"]);
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            shell_command(
                "test \"$name\" != backend",
                "if %name%==backend (exit /b 1) else (exit /b 0)"
            )
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            "fatal: command failed in 'backend' with exit status 1\nfatal: \
             foreach failed\n"
        );
}

#[test]
fn foreach_reports_multiple_failures_in_repository_order()
{
    let tmp = create_vmr(&["alpha", "zeta"]);
    create_child_repo(tmp.path(), "alpha");
    create_child_repo(tmp.path(), "zeta");

    let output = git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", shell_command("false", "exit /b 1")])
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
    let tmp = create_vmr(&["backend", "frontend"]);
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            shell_command(
                "test \"$name\" = frontend",
                "if %name%==frontend (exit /b 0) else (exit /b 1)"
            )
        ])
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
    let tmp = create_vmr(&["backend"]);
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            shell_command(
                "printf '%s;%s;%s;%s;%s\\n' \"$name\" \"$sm_path\" \"$displaypath\" \"$toplevel\" \"${sha1-unset}\"",
                "if defined sha1 (exit /b 1) else echo %name%;%sm_path%;%displaypath%;%toplevel%;unset"
            )
        ])
        .assert()
        .success()
        .stdout(format!(
            ".;.;.;{toplevel};unset{CHILD_NEWLINE}\
             backend;backend;backend;{toplevel};unset{CHILD_NEWLINE}",
            toplevel = shown_dir(tmp.path()).display()
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_sets_name_to_the_manifest_name_and_sm_path_to_the_path()
{
    // Arrange: the manifest names `backend` differently from its path
    let tmp = create_vmr(&[]);
    fs::write(
        tmp.path().join(".gitvmr"),
        "[repo.api]\npath = \"backend\"\nurl = \"../api.git\"\n"
    )
    .expect("failed to write manifest");
    fs::write(tmp.path().join(".gitignore"), "/backend/\n")
        .expect("failed to write .gitignore");
    create_child_repo(tmp.path(), "backend");

    // Act
    let assert = git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            "--quiet",
            shell_command(
                "printf '%s;%s\\n' \"$name\" \"$sm_path\"",
                "echo %name%;%sm_path%"
            )
        ])
        .assert();

    // Assert
    assert
        .success()
        .stdout(child_lines(&[".;.", "api;backend"]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_displaypath_respects_effective_working_directory()
{
    let tmp = create_vmr(&["backend", "frontend"]);
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");

    git_vmr()
        .current_dir(tmp.path().join("frontend"))
        .args([
            "foreach",
            "--quiet",
            shell_command(
                "echo \"$name:$displaypath\"",
                "echo %name%:%displaypath%"
            )
        ])
        .assert()
        .success()
        .stdout(child_lines(&[
            ".:..",
            if cfg!(windows)
            {
                r"backend:..\backend"
            }
            else
            {
                "backend:../backend"
            },
            "frontend:."
        ]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_closes_child_stdin()
{
    let tmp = create_vmr(&["backend"]);
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
    let tmp = create_vmr(&["backend", "frontend"]);
    create_child_repo(tmp.path(), "backend");
    create_child_repo(tmp.path(), "frontend");
    fs::create_dir_all(tmp.path().join("frontend/src"))
        .expect("failed to create nested dir");

    git_vmr()
        .arg("-C")
        .arg(tmp.path().join("frontend/src"))
        .args([
            "foreach",
            "--quiet",
            shell_command("printf '%s\\n' \"$name\"", "echo %name%")
        ])
        .assert()
        .success()
        .stdout(child_lines(&[".", "backend", "frontend"]))
        .stderr(predicate::str::is_empty());
}
