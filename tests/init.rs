use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn git_vmr() -> Command
{
    Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary")
}

#[test]
fn init_creates_default_config_in_current_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .gitvmr in"))
        .stderr(predicate::str::is_empty());

    let config_path = tmp.path().join(".gitvmr/config");
    let config =
        fs::read_to_string(config_path).expect("failed to read config");
    assert!(config.contains("[core]"));
    assert!(config.contains("version = 0"));
}

#[test]
fn init_with_working_dir_argument_creates_config_in_requested_directory()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let target = tmp.path().join("target");
    fs::create_dir(&target).expect("failed to create target dir");

    git_vmr()
        .arg("-C")
        .arg(&target)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .gitvmr in"))
        .stderr(predicate::str::is_empty());

    assert!(target.join(".gitvmr/config").is_file());
    assert!(!tmp.path().join(".gitvmr").exists());
}

#[test]
fn init_with_existing_directory_argument_creates_config_in_target()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let target = tmp.path().join("project");
    fs::create_dir(&target).expect("failed to create target dir");

    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .arg("project")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .gitvmr in"))
        .stderr(predicate::str::is_empty());

    assert!(target.join(".gitvmr/config").is_file());
    assert!(!tmp.path().join(".gitvmr").exists());
}

#[test]
fn init_with_missing_directory_argument_creates_target_and_config()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let target = tmp.path().join("project");

    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .arg("project")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .gitvmr in"))
        .stderr(predicate::str::is_empty());

    assert!(target.is_dir());
    assert!(target.join(".gitvmr/config").is_file());
}

#[test]
fn init_directory_argument_is_relative_to_working_dir_argument()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let base = tmp.path().join("base");
    fs::create_dir(&base).expect("failed to create base dir");

    git_vmr()
        .arg("-C")
        .arg(&base)
        .arg("init")
        .arg("project")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .gitvmr in"))
        .stderr(predicate::str::is_empty());

    assert!(base.join("project/.gitvmr/config").is_file());
    assert!(!tmp.path().join("project").exists());
}

#[test]
fn init_errors_when_directory_argument_is_file()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let file = tmp.path().join("file");
    fs::write(&file, "").expect("failed to write file");

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

    assert!(!file.join(".gitvmr/config").exists());
}

#[test]
fn init_reports_reinitialized_without_overwriting_existing_config()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join(".gitvmr");
    let config_path = config_dir.join("config");
    fs::create_dir_all(&config_dir).expect("failed to create config dir");
    fs::write(&config_path, "custom content").expect("failed to write config");

    git_vmr()
        .current_dir(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Reinitialized existing virtual monorepo in"
        ))
        .stderr(predicate::str::is_empty());

    let config =
        fs::read_to_string(config_path).expect("failed to read config");
    assert_eq!(config, "custom content");
}

#[test]
fn init_errors_when_working_dir_argument_does_not_exist()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let missing = tmp.path().join("missing");

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

    assert!(!tmp.path().join(".gitvmr").exists());
}

#[test]
fn version_flag_prints_package_version()
{
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
