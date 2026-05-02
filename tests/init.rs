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
