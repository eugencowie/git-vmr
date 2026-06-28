mod common;

use assert_cmd::Command;
use common::git_vmr;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn init_vmr(path: &Path)
{
    fs::create_dir(path.join(".gitvmr")).expect("failed to create marker");
}

fn git_vmr_with_state(config_dir: &Path, state_dir: &Path) -> Command
{
    let config_file = config_dir.join("git-vmr").join("config.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(
        &config_file,
        "[core]\nversion = 0\n\n[updates]\ncheckfrequency = \"never\"\n"
    )
    .expect("failed to write config");

    let mut command =
        Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary");
    command
        .env("GITVMR_CONFIG_DIR", config_dir)
        .env("GITVMR_STATE_DIR", state_dir);
    command
}

fn state_file(state_dir: &Path) -> PathBuf
{
    state_dir.join("git-vmr").join("state.toml")
}

fn read_session_id(state_dir: &Path) -> String
{
    let contents =
        fs::read_to_string(state_file(state_dir)).expect("expected state file");
    let state: toml::Value =
        toml::from_str(&contents).expect("expected state toml");
    state["analytics"]["session_id"]
        .as_str()
        .expect("expected analytics session id")
        .to_owned()
}

fn read_event(path: &Path) -> Value
{
    let events = read_events(path);
    assert_eq!(events.len(), 1);
    events.into_iter().next().unwrap()
}

fn read_events(path: &Path) -> Vec<Value>
{
    fs::read_to_string(path)
        .expect("expected analytics event log")
        .lines()
        .map(|line| serde_json::from_str(line).expect("expected event json"))
        .collect()
}

#[test]
fn successful_dispatched_command_records_analytics()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .current_dir(&vmr)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let event = read_event(&log);
    assert_eq!(event["name"], "status");
    assert_eq!(event["success"], true);
    assert!(event.get("sessionId").is_none());
}

#[test]
fn failed_dispatched_command_records_analytics_and_preserves_error()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .current_dir(tmp.path())
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("fatal: not a virtual monorepo"));

    let event = read_event(&log);
    assert_eq!(event["name"], "status");
    assert_eq!(event["success"], false);
    assert!(event.get("sessionId").is_none());
}

#[test]
fn consecutive_dispatched_commands_reuse_analytics_session()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let log = tmp.path().join("analytics.jsonl");

    git_vmr_with_state(&config_dir, &state_dir)
        .current_dir(&vmr)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .success();
    let first_session = read_session_id(&state_dir);

    git_vmr_with_state(&config_dir, &state_dir)
        .current_dir(&vmr)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .success();
    let second_session = read_session_id(&state_dir);

    let events = read_events(&log);
    assert_eq!(events.len(), 2);
    assert_eq!(first_session, second_session);
}

#[test]
fn failed_dispatched_commands_reuse_session_and_preserve_error()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr_with_state(&config_dir, &state_dir)
        .current_dir(tmp.path())
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("fatal: not a virtual monorepo"));
    let first_session = read_session_id(&state_dir);

    git_vmr_with_state(&config_dir, &state_dir)
        .current_dir(tmp.path())
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("fatal: not a virtual monorepo"));
    let second_session = read_session_id(&state_dir);

    let events = read_events(&log);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["success"], false);
    assert_eq!(first_session, second_session);
}

#[test]
fn analytics_payload_records_flag_names_only()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .current_dir(tmp.path())
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("-C")
        .arg(&vmr)
        .args(["foreach", "--quiet", "echo", "secret"])
        .assert()
        .success();

    let event = read_event(&log);
    assert_eq!(event["name"], "foreach");
    assert_eq!(event["quiet_flag"], true);
    assert_eq!(event["working_dir_global_flag"], true);
    let event_strings =
        event.as_object().unwrap().values().filter_map(Value::as_str);
    assert!(!event_strings.clone().any(|value| value == "secret"));
    assert!(!event_strings.clone().any(|value| value == vmr.to_str().unwrap()));
}

#[test]
fn disabled_analytics_skips_event()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let vmr = tmp.path().join("vmr");
    fs::create_dir(&vmr).expect("failed to create vmr dir");
    init_vmr(&vmr);
    let log = tmp.path().join("analytics.jsonl");
    let config_dir = tmp.path().join("config");
    let config_file = config_dir.join("git-vmr").join("config.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(
        &config_file,
        "[updates]\ncheckfrequency = \"never\"\n\n[analytics]\nenabled = false\n"
    )
    .expect("failed to write config");

    git_vmr()
        .current_dir(&vmr)
        .env("GITVMR_CONFIG_DIR", &config_dir)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("status")
        .assert()
        .success();

    assert!(!log.exists());
}

#[test]
fn version_output_does_not_record_analytics()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("--version")
        .assert()
        .success();

    assert!(!log.exists());
}

#[test]
fn excluded_command_paths_do_not_create_analytics_session_state()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr_with_state(&config_dir, &state_dir)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("--version")
        .assert()
        .success();
    git_vmr_with_state(&config_dir, &state_dir)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("--help")
        .assert()
        .success();
    git_vmr_with_state(&config_dir, &state_dir)
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("--not-a-real-flag")
        .assert()
        .failure();
    git_vmr_with_state(&config_dir, &state_dir)
        .current_dir(tmp.path())
        .env("GITVMR_ANALYTICS_LOG", &log)
        .args(["-C", "missing", "status"])
        .assert()
        .failure();

    assert!(!log.exists());
    assert!(!state_file(&state_dir).exists());
}

#[test]
fn help_output_does_not_record_analytics()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("--help")
        .assert()
        .success();

    assert!(!log.exists());
}

#[test]
fn parse_error_does_not_record_analytics()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .env("GITVMR_ANALYTICS_LOG", &log)
        .arg("--not-a-real-flag")
        .assert()
        .failure();

    assert!(!log.exists());
}

#[test]
fn context_construction_failure_does_not_record_analytics()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let log = tmp.path().join("analytics.jsonl");

    git_vmr()
        .current_dir(tmp.path())
        .env("GITVMR_ANALYTICS_LOG", &log)
        .args(["-C", "missing", "status"])
        .assert()
        .failure();

    assert!(!log.exists());
}
