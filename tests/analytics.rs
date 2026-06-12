mod common;

use common::git_vmr;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;

fn init_vmr(path: &Path)
{
    fs::create_dir(path.join(".gitvmr")).expect("failed to create marker");
}

fn read_event(path: &Path) -> Value
{
    let contents =
        fs::read_to_string(path).expect("expected analytics event log");
    serde_json::from_str(contents.trim()).expect("expected event json")
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
    assert!(!event.to_string().contains("secret"));
    assert!(!event.to_string().contains(vmr.to_str().unwrap()));
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
