mod common;

use common::git_vmr;
use predicates::prelude::*;
use std::fs;

#[test]
fn malformed_global_config_fails_before_subcommand_runs()
{
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let config_dir = tmp.path().join("config");
    let config_file = config_dir.join("git-vmr").join("config.toml");
    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create config dir");
    fs::write(&config_file, "[core]\nversion = true\n")
        .expect("failed to write config");

    git_vmr()
        .current_dir(tmp.path())
        .env("GIT_VMR_CONFIG_DIR", &config_dir)
        .arg("status")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("failed to parse"))
        .stderr(predicate::str::contains("not a virtual monorepo").not());
}
