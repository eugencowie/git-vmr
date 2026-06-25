mod common;

use common::git_vmr;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

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
    let state_file = state_dir.join("git-vmr").join("state.toml");

    git_vmr()
        .current_dir(&vmr)
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let state = fs::read_to_string(state_file)
        .expect("expected update-check state to be written");
    assert!(state.contains("[updates]"));
    assert!(state.contains("last_check"));
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
    let state_file = state_dir.join("git-vmr").join("state.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(&config_file, "[updates]\ncheckfrequency = \"never\"\n")
        .expect("failed to write config");

    git_vmr()
        .current_dir(&vmr)
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let state = fs::read_to_string(state_file)
        .expect("expected analytics state to be written");
    assert!(!state.contains("last_check"));
}

#[test]
fn invalid_update_state_warns_and_uses_defaults()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let state_file = state_dir.join("git-vmr").join("state.toml");
    fs::create_dir_all(state_file.parent().unwrap())
        .expect("failed to create state dir");
    fs::write(&state_file, "[updates]\nlast_check =")
        .expect("failed to write state");

    git_vmr()
        .current_dir(&vmr)
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("warning: failed to parse"))
        .stderr(predicate::str::contains("using default state"));

    let state = fs::read_to_string(state_file)
        .expect("expected invalid state to be replaced");
    assert!(state.contains("last_check"));
}

#[test]
fn invalid_global_config_value_fails_before_subcommand_runs()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let config_file = config_dir.join("git-vmr").join("config.toml");
    let state_file = state_dir.join("git-vmr").join("state.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(&config_file, "[updates]\ncheckfrequency = \"daily\"\n")
        .expect("failed to write config");

    git_vmr()
        .current_dir(tmp.path())
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to parse"))
        .stderr(predicate::str::contains("unsupported frequency"))
        .stderr(predicate::str::contains("not a virtual monorepo").not());

    assert!(!state_file.exists());
}

#[test]
fn syntax_error_in_global_config_fails_before_subcommand_runs()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let config_file = config_dir.join("git-vmr").join("config.toml");
    let state_file = state_dir.join("git-vmr").join("state.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(&config_file, "[updates\ncheck_frequency = \"1 day\"\n")
        .expect("failed to write config");

    git_vmr()
        .current_dir(tmp.path())
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to parse"))
        .stderr(predicate::str::contains("not a virtual monorepo").not());

    assert!(!state_file.exists());
}

#[test]
fn failed_subcommand_still_runs_update_check()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let state_file = state_dir.join("git-vmr").join("state.toml");

    git_vmr()
        .current_dir(tmp.path())
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("fatal: not a virtual monorepo"));

    let state = fs::read_to_string(state_file)
        .expect("expected end-of-run state to be written");
    assert!(state.contains("[updates]"));
    assert!(state.contains("last_check"));
}

#[test]
fn version_output_does_not_trigger_update_check()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let state_file = state_dir.join("git-vmr").join("state.toml");

    git_vmr()
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_STATE_DIR", &state_dir)
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
        .env("GITVMR_CONFIG_DIR", tmp.path().join("config"))
        .env("GITVMR_STATE_DIR", tmp.path().join("state"))
        .env("AXOUPDATER_CONFIG_PATH", tmp.path().join("receipt"))
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}
