use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_CONFIG_DIR_ID: AtomicUsize = AtomicUsize::new(0);

pub fn git_vmr() -> Command
{
    let config_dir = isolated_config_dir();
    let config_file = config_dir.join("git-vmr").join("config.toml");

    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create isolated global config dir");
    fs::write(&config_file, "[core]\nversion = 0\n")
        .expect("failed to write isolated global config");

    let mut command =
        Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary");
    command.env("GIT_VMR_CONFIG_DIR", config_dir);
    command
}

fn isolated_config_dir() -> PathBuf
{
    let id = NEXT_CONFIG_DIR_ID.fetch_add(1, Ordering::Relaxed);

    std::env::temp_dir()
        .join("git-vmr-tests")
        .join(format!("{}-{id}", std::process::id()))
        .join("config")
}
