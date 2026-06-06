use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn git_vmr() -> Command
{
    Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary")
}

fn init_vmr(path: &Path)
{
    fs::create_dir(path.join(".gitvmr")).expect("failed to create marker");
}

#[test]
fn successful_subcommand_triggers_due_update_check()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let state_file = state_dir.join("git-vmr").join("update.toml");

    git_vmr()
        .current_dir(&vmr)
        .env("GIT_VMR_CONFIG_DIR", &config_dir)
        .env("GIT_VMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let state = fs::read_to_string(state_file)
        .expect("expected update-check state to be written");
    assert!(state.contains("last_attempted_check"));
}

#[test]
fn configured_never_through_global_config_skips_update_check()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let config_file = config_dir.join("git-vmr").join("config.toml");
    let state_file = state_dir.join("git-vmr").join("update.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(&config_file, "[updates]\ncheckfrequency = \"never\"\n")
        .expect("failed to write config");

    git_vmr()
        .current_dir(&vmr)
        .env("GIT_VMR_CONFIG_DIR", &config_dir)
        .env("GIT_VMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(!state_file.exists());
}

#[test]
fn failed_subcommand_does_not_trigger_update_check()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let state_file = state_dir.join("git-vmr").join("update.toml");

    git_vmr()
        .current_dir(tmp.path())
        .env("GIT_VMR_CONFIG_DIR", &config_dir)
        .env("GIT_VMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("fatal: not a virtual monorepo"));

    assert!(!state_file.exists());
}

#[test]
fn version_output_does_not_trigger_update_check()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let state_file = state_dir.join("git-vmr").join("update.toml");

    git_vmr()
        .env("GIT_VMR_CONFIG_DIR", &config_dir)
        .env("GIT_VMR_STATE_DIR", &state_dir)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("git-vmr"))
        .stderr(predicate::str::is_empty());

    assert!(!state_file.exists());
}

#[test]
fn missing_receipt_update_failure_does_not_fail_successful_command()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);

    git_vmr()
        .current_dir(&vmr)
        .env("GIT_VMR_CONFIG_DIR", tmp.path().join("config"))
        .env("GIT_VMR_STATE_DIR", tmp.path().join("state"))
        .env("AXOUPDATER_CONFIG_PATH", tmp.path().join("receipt"))
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}
