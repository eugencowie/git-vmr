mod common;

use common::git_vmr;
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

#[cfg(unix)]
fn expected_current_dir(path: &Path) -> std::path::PathBuf
{
    path.canonicalize().unwrap()
}

#[cfg(windows)]
fn expected_current_dir(path: &Path) -> std::path::PathBuf
{
    // `CliContext` gets the process working directory from GetCurrentDirectory,
    // which preserves the non-verbatim spelling passed to `current_dir`.
    path.to_path_buf()
}

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
        .args([
            "foreach",
            "--quiet",
            shell_command("printf '%s\\n' \"$name\"", "echo %name%")
        ])
        .assert()
        .success()
        .stdout(child_lines(&["backend", "frontend"]))
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
            shell_command(
                "if [ \"$name\" = zeta ]; then sleep 0.2; fi; echo \"$name\"",
                "if %name%==zeta (ping -n 2 127.0.0.1 >NUL&echo %name%) else echo %name%"
            )
        ])
        .assert()
        .success()
        .stdout(child_lines(&["alpha", "zeta"]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_replays_stdout_stderr_and_default_headers()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args([
            "foreach",
            shell_command("echo out; echo err >&2", "echo out&echo err>&2")
        ])
        .assert()
        .success()
        .stdout(format!("Entering 'backend'\nout{CHILD_NEWLINE}"))
        .stderr(format!("err{CHILD_NEWLINE}"));
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
        .stdout(child_lines(&["ok"]))
        .stderr(predicate::str::is_empty());
}

#[test]
fn foreach_reports_single_child_failure()
{
    let tmp = create_vmr();
    create_child_repo(tmp.path(), "backend");

    git_vmr()
        .current_dir(tmp.path())
        .args(["foreach", "--quiet", shell_command("false", "exit /b 1")])
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
    let tmp = create_vmr();
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
    let tmp = create_vmr();
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
            "backend;backend;backend;{};unset{}",
            expected_current_dir(tmp.path()).display(),
            CHILD_NEWLINE
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
        .args([
            "foreach",
            "--quiet",
            shell_command("printf '%s\\n' \"$name\"", "echo %name%")
        ])
        .assert()
        .success()
        .stdout(child_lines(&["backend", "frontend"]))
        .stderr(predicate::str::is_empty());
}
