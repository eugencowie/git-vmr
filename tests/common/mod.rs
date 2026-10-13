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

/// Disables Git's auto-maintenance, which otherwise runs detached after
/// commits and briefly creates lock files in `.git` that race with tests
/// copying or inspecting repositories.
const NO_AUTO_MAINTENANCE: [(&str, &str); 3] = [
    ("GIT_CONFIG_COUNT", "1"),
    ("GIT_CONFIG_KEY_0", "maintenance.auto"),
    ("GIT_CONFIG_VALUE_0", "false")
];

/// Git's repository-local environment variables, as reported by git itself.
/// Git exports these when it spawns subprocesses (e.g. `git rebase --exec`);
/// if inherited, they redirect every git operation from the test fixture
/// repositories to the developer's real repository.
fn git_env_overrides() -> &'static [String]
{
    static VARS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

    VARS.get_or_init(|| {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--local-env-vars"])
            .output()
            .expect("failed to run git rev-parse --local-env-vars");
        assert!(
            output.status.success(),
            "git rev-parse --local-env-vars failed"
        );
        String::from_utf8(output.stdout)
            .expect("git env var names should be UTF-8")
            .lines()
            .map(str::to_owned)
            .collect()
    })
}

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
    for var in git_env_overrides()
    {
        command.env_remove(var);
    }
    command.envs(NO_AUTO_MAINTENANCE);
    command
}

pub fn git<const N: usize>(dir: &Path, args: [&str; N])
{
    run_git(dir, args);
}

/// A `git` command with the fixture-safe environment: fixed dates, no
/// auto-maintenance, and no inherited `GIT_*` overrides. All test git
/// invocations must start here.
pub fn git_command() -> std::process::Command
{
    let mut command = std::process::Command::new("git");
    command
        .env("GIT_AUTHOR_DATE", TEST_GIT_DATE)
        .env("GIT_COMMITTER_DATE", TEST_GIT_DATE);
    for var in git_env_overrides()
    {
        command.env_remove(var);
    }
    command.envs(NO_AUTO_MAINTENANCE);
    command
}

fn run_git<const N: usize>(dir: &Path, args: [&str; N])
-> std::process::Output
{
    let output = git_command()
        .args(args)
        .current_dir(dir)
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

/// Makes `path` a VMR root: a root repo declaring `repos` (see [`declare`]).
/// The child repos themselves are created separately, so a declared name
/// without a matching [`init_repo`] is a child repo that is not cloned.
pub fn init_vmr(path: &Path, repos: &[&str])
{
    git(path, ["init", "--quiet", "--initial-branch=master"]);
    declare(path, repos);
}

/// Rewrites the VMR root's manifest to declare exactly `repos` (name =
/// path, with `file://` URLs) and its `.gitignore` to ignore them.
pub fn declare(path: &Path, repos: &[&str])
{
    let manifest = repos
        .iter()
        .map(|repo| {
            let url = format!("file:///remotes/{repo}.git");
            format!("[repo.\"{repo}\"]\npath = \"{repo}\"\nurl = \"{url}\"\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path.join(".gitvmr"), manifest)
        .expect("failed to write manifest");
    let gitignore =
        repos.iter().map(|repo| format!("/{repo}/\n")).collect::<String>();
    fs::write(path.join(".gitignore"), gitignore)
        .expect("failed to write .gitignore");
}

/// How the CLI shows the directory it was run from: the process working dir,
/// which resolves symlinks such as macOS's `/var` on Unix.
#[cfg(unix)]
pub fn shown_dir(path: &Path) -> PathBuf
{
    path.canonicalize().expect("failed to canonicalize path")
}

/// How the CLI shows the directory it was run from: GetCurrentDirectory
/// preserves the spelling passed to `current_dir`.
#[cfg(windows)]
pub fn shown_dir(path: &Path) -> PathBuf
{
    path.to_path_buf()
}

pub fn init_repo(path: &Path)
{
    fs::create_dir(path).expect("failed to create repo dir");
    git(path, ["init", "--initial-branch=master"]);
    git(path, ["config", "core.autocrlf", "false"]);
    git(path, ["config", "user.email", "test@example.com"]);
    git(path, ["config", "user.name", "Test User"]);
}

pub fn clone_repo(source: &Path, destination: &Path)
{
    let parent = destination.parent().expect("clone destination has parent");
    git(parent, [
        "-c",
        "core.autocrlf=false",
        "clone",
        source.to_str().expect("source path should be UTF-8"),
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .expect("destination name should be UTF-8")
    ]);
    git(destination, ["config", "core.autocrlf", "false"]);
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
/// a root repo declaring one committed child repo per requested name.
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
        init_vmr(&vmr, repos);

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
