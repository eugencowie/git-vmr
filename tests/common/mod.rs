// Each integration-test binary compiles this module separately and uses
// only a subset of the fixtures, so unused-item lints are expected here.
#![allow(dead_code)]

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

static NEXT_TEST_ENV_ID: AtomicUsize = AtomicUsize::new(0);
const TEST_GIT_DATE: &str = "2000-01-01T00:00:00Z";

pub fn git_vmr() -> Command
{
    let test_env = isolated_test_env();
    let config_dir = test_env.join("config");
    let state_dir = test_env.join("state");
    let config_file = config_dir.join("git-vmr").join("config.toml");

    fs::create_dir_all(config_file.parent().unwrap())
        .expect("failed to create isolated global config dir");
    fs::write(
        &config_file,
        "[core]\nversion = 0\n\n[updates]\ncheckfrequency = \"never\"\n"
    )
    .expect("failed to write isolated global config");

    let mut command =
        Command::cargo_bin("git-vmr").expect("failed to find git-vmr binary");
    command
        .env("GITVMR_CONFIG_DIR", config_dir)
        .env("GITVMR_STATE_DIR", state_dir);
    command
}

pub fn git<const N: usize>(dir: &Path, args: [&str; N])
{
    run_git(dir, args);
}

fn run_git<const N: usize>(dir: &Path, args: [&str; N])
-> std::process::Output
{
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", TEST_GIT_DATE)
        .env("GIT_COMMITTER_DATE", TEST_GIT_DATE)
        .output()
        .expect("failed to run git");
    assert!(
        output.status.success(),
        "git {:?} failed in '{}'\nstderr: {}\nstdout: {}",
        args,
        dir.display(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    output
}

pub fn git_output<const N: usize>(dir: &Path, args: [&str; N]) -> String
{
    let output = run_git(dir, args);

    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

pub fn init_vmr(path: &Path)
{
    fs::create_dir(path.join(".gitvmr")).expect("failed to create marker");
}

pub fn init_repo(path: &Path)
{
    fs::create_dir(path).expect("failed to create repo dir");
    git(path, ["init"]);
    git(path, ["config", "user.email", "test@example.com"]);
    git(path, ["config", "user.name", "Test User"]);
}

pub fn clone_repo(source: &Path, destination: &Path)
{
    let parent = destination.parent().expect("clone destination has parent");
    git(parent, [
        "clone",
        source.to_str().expect("source path should be UTF-8"),
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .expect("destination name should be UTF-8")
    ]);
    git(destination, ["config", "user.email", "test@example.com"]);
    git(destination, ["config", "user.name", "Test User"]);
}

pub fn write_commit(path: &Path, file: &str, content: &str, message: &str)
{
    if let Some(parent) = path.join(file).parent()
    {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path.join(file), content).expect("failed to write file");
    git(path, ["add", file]);
    git(path, ["commit", "-m", message]);
}

pub fn commit_file(path: &Path, file: &str)
{
    write_commit(path, file, "content\n", "initial");
}

/// A materialized VMR fixture: a tempdir holding a `vmr` subdirectory with
/// the `.gitvmr` marker and one committed child repo per requested name.
///
/// Owning the [`TempDir`] keeps the fixture alive for the test's duration;
/// the VMR sits in a subdirectory so worktree roots can materialize as
/// siblings via [`TestVmr::sibling`].
pub struct TestVmr
{
    tmp: TempDir,
    vmr: PathBuf
}

impl TestVmr
{
    pub fn with_repos(repos: &[&str]) -> Self
    {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let vmr = tmp.path().join("vmr");
        fs::create_dir(&vmr).expect("failed to create vmr dir");
        init_vmr(&vmr);

        for repo in repos
        {
            let repo_path = vmr.join(repo);
            init_repo(&repo_path);
            commit_file(&repo_path, "README.md");
        }

        Self { tmp, vmr }
    }

    /// The VMR root.
    pub fn path(&self) -> &Path
    {
        &self.vmr
    }

    /// The path of a child repo inside the VMR.
    pub fn repo(&self, name: &str) -> PathBuf
    {
        self.vmr.join(name)
    }

    /// A tempdir-level path next to the VMR root.
    pub fn sibling(&self, name: &str) -> PathBuf
    {
        self.tmp.path().join(name)
    }
}

fn isolated_test_env() -> PathBuf
{
    let id = NEXT_TEST_ENV_ID.fetch_add(1, Ordering::Relaxed);

    std::env::temp_dir()
        .join("git-vmr-tests")
        .join(format!("{}-{id}", std::process::id()))
}
