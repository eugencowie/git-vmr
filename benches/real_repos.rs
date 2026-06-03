use criterion::{Criterion, criterion_group, criterion_main};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, fs, io};

const FIXTURE_ENV_VAR: &str = "GIT_VMR_BENCH_FIXTURE_DIR";
const DEFAULT_FIXTURE_DIR: &str = "target/bench-fixtures/real-repos";

#[derive(Debug)]
struct Repo
{
    name: &'static str,
    url: &'static str,
    revision: &'static str
}

const REPOS: &[Repo] = &[
    Repo {
        name: "ripgrep",
        url: "https://github.com/BurntSushi/ripgrep.git",
        revision: "14.1.1"
    },
    Repo {
        name: "fd",
        url: "https://github.com/sharkdp/fd.git",
        revision: "v10.3.0"
    },
    Repo {
        name: "bat",
        url: "https://github.com/sharkdp/bat.git",
        revision: "v0.25.0"
    },
    Repo {
        name: "exa",
        url: "https://github.com/ogham/exa.git",
        revision: "v0.10.1"
    },
    Repo {
        name: "hyperfine",
        url: "https://github.com/sharkdp/hyperfine.git",
        revision: "v1.19.0"
    },
    Repo {
        name: "tokei",
        url: "https://github.com/XAMPPRocky/tokei.git",
        revision: "v12.1.2"
    },
    Repo {
        name: "zoxide",
        url: "https://github.com/ajeetdsouza/zoxide.git",
        revision: "v0.9.8"
    },
    Repo {
        name: "starship",
        url: "https://github.com/starship/starship.git",
        revision: "v1.23.0"
    },
    Repo {
        name: "git-delta",
        url: "https://github.com/dandavison/delta.git",
        revision: "0.18.2"
    },
    Repo {
        name: "bottom",
        url: "https://github.com/ClementTsang/bottom.git",
        revision: "0.10.2"
    },
    Repo {
        name: "nushell",
        url: "https://github.com/nushell/nushell.git",
        revision: "0.104.1"
    },
    Repo {
        name: "helix",
        url: "https://github.com/helix-editor/helix.git",
        revision: "25.01.1"
    },
    Repo {
        name: "alacritty",
        url: "https://github.com/alacritty/alacritty.git",
        revision: "v0.15.1"
    },
    Repo {
        name: "deno",
        url: "https://github.com/denoland/deno.git",
        revision: "v2.2.8"
    },
    Repo {
        name: "ruff",
        url: "https://github.com/astral-sh/ruff.git",
        revision: "0.11.2"
    },
    Repo {
        name: "uv",
        url: "https://github.com/astral-sh/uv.git",
        revision: "0.6.9"
    },
    Repo {
        name: "rye",
        url: "https://github.com/astral-sh/rye.git",
        revision: "0.44.0"
    },
    Repo {
        name: "maturin",
        url: "https://github.com/PyO3/maturin.git",
        revision: "v1.8.3"
    },
    Repo {
        name: "pydantic",
        url: "https://github.com/pydantic/pydantic.git",
        revision: "v2.11.1"
    },
    Repo {
        name: "fastapi",
        url: "https://github.com/fastapi/fastapi.git",
        revision: "0.115.12"
    },
    Repo {
        name: "flask",
        url: "https://github.com/pallets/flask.git",
        revision: "3.1.0"
    },
    Repo {
        name: "requests",
        url: "https://github.com/psf/requests.git",
        revision: "v2.32.3"
    },
    Repo {
        name: "pytest",
        url: "https://github.com/pytest-dev/pytest.git",
        revision: "8.3.5"
    },
    Repo {
        name: "black",
        url: "https://github.com/psf/black.git",
        revision: "25.1.0"
    },
    Repo {
        name: "django",
        url: "https://github.com/django/django.git",
        revision: "5.1.7"
    },
    Repo {
        name: "prometheus",
        url: "https://github.com/prometheus/prometheus.git",
        revision: "v3.2.1"
    },
    Repo {
        name: "terraform",
        url: "https://github.com/hashicorp/terraform.git",
        revision: "v1.11.2"
    },
    Repo {
        name: "vue",
        url: "https://github.com/vuejs/core.git",
        revision: "v3.5.13"
    },
    Repo {
        name: "svelte",
        url: "https://github.com/sveltejs/svelte.git",
        revision: "svelte@5.25.3"
    },
    Repo {
        name: "vite",
        url: "https://github.com/vitejs/vite.git",
        revision: "v6.2.3"
    },
    Repo {
        name: "webpack",
        url: "https://github.com/webpack/webpack.git",
        revision: "v5.98.0"
    },
    Repo {
        name: "eslint",
        url: "https://github.com/eslint/eslint.git",
        revision: "v9.23.0"
    },
    Repo {
        name: "prettier",
        url: "https://github.com/prettier/prettier.git",
        revision: "3.5.3"
    },
    Repo {
        name: "tailwindcss",
        url: "https://github.com/tailwindlabs/tailwindcss.git",
        revision: "v4.0.17"
    },
    Repo {
        name: "bootstrap",
        url: "https://github.com/twbs/bootstrap.git",
        revision: "v5.3.3"
    },
    Repo {
        name: "xsv",
        url: "https://github.com/BurntSushi/xsv.git",
        revision: "0.13.0"
    },
    Repo {
        name: "hexyl",
        url: "https://github.com/sharkdp/hexyl.git",
        revision: "v0.16.0"
    },
    Repo {
        name: "pastel",
        url: "https://github.com/sharkdp/pastel.git",
        revision: "v0.10.0"
    },
    Repo {
        name: "just",
        url: "https://github.com/casey/just.git",
        revision: "1.40.0"
    },
    Repo {
        name: "diskus",
        url: "https://github.com/sharkdp/diskus.git",
        revision: "v0.8.0"
    },
    Repo {
        name: "numbat",
        url: "https://github.com/sharkdp/numbat.git",
        revision: "v1.16.0"
    },
    Repo {
        name: "vivid",
        url: "https://github.com/sharkdp/vivid.git",
        revision: "v0.10.1"
    },
    Repo {
        name: "dust",
        url: "https://github.com/bootandy/dust.git",
        revision: "v1.1.1"
    },
    Repo {
        name: "lsd",
        url: "https://github.com/lsd-rs/lsd.git",
        revision: "v1.1.5"
    },
    Repo {
        name: "procs",
        url: "https://github.com/dalance/procs.git",
        revision: "v0.14.10"
    },
    Repo {
        name: "duf",
        url: "https://github.com/muesli/duf.git",
        revision: "v0.8.1"
    },
    Repo {
        name: "gdu",
        url: "https://github.com/dundee/gdu.git",
        revision: "v5.31.0"
    },
    Repo {
        name: "skim",
        url: "https://github.com/lotabout/skim.git",
        revision: "v0.16.1"
    },
    Repo {
        name: "amber",
        url: "https://github.com/dalance/amber.git",
        revision: "v0.6.0"
    },
    Repo {
        name: "broot",
        url: "https://github.com/Canop/broot.git",
        revision: "v1.44.5"
    }
];

fn status_real_repos(c: &mut Criterion)
{
    let fixture_root =
        fixture_root().expect("failed to determine benchmark fixture root");
    validate_manifest().expect("benchmark repository manifest is invalid");
    prepare_fixture(&fixture_root)
        .expect("failed to prepare status benchmark fixture");
    validate_fixture(&fixture_root)
        .expect("status benchmark fixture is invalid");

    let binary = git_vmr_binary().expect("failed to locate git-vmr binary");
    run_status(&binary, &fixture_root, "warmup")
        .expect("status benchmark warmup failed");

    c.bench_function(
        "status_real_repos_warm_cache_full_command_wall_time",
        |b| {
            b.iter(|| {
                run_status(&binary, &fixture_root, "measured")
                    .expect("status benchmark command failed")
            });
        }
    );
}

fn branch_real_repos(c: &mut Criterion)
{
    let fixture_root =
        fixture_root().expect("failed to determine benchmark fixture root");
    validate_manifest().expect("benchmark repository manifest is invalid");
    prepare_fixture(&fixture_root)
        .expect("failed to prepare branch benchmark fixture");
    validate_fixture(&fixture_root)
        .expect("branch benchmark fixture is invalid");

    let binary = git_vmr_binary().expect("failed to locate git-vmr binary");
    run_branch(&binary, &fixture_root, "warmup")
        .expect("branch benchmark warmup failed");

    c.bench_function(
        "branch_real_repos_warm_cache_full_command_wall_time",
        |b| {
            b.iter(|| {
                run_branch(&binary, &fixture_root, "measured")
                    .expect("branch benchmark command failed")
            });
        }
    );
}

fn fixture_root() -> io::Result<PathBuf>
{
    match env::var_os(FIXTURE_ENV_VAR)
    {
        Some(path) => Ok(PathBuf::from(path)),
        None => env::current_dir().map(|cwd| cwd.join(DEFAULT_FIXTURE_DIR))
    }
}

fn validate_manifest() -> Result<(), String>
{
    for repo in REPOS
    {
        if repo.name.is_empty()
            || repo.name == "."
            || repo.name == ".."
            || repo.name.contains('/')
            || repo.name.contains('\\')
            || repo.name.contains(std::path::MAIN_SEPARATOR)
        {
            return Err(format!(
                "invalid benchmark repository directory name '{}'",
                repo.name
            ));
        }
    }

    Ok(())
}

fn prepare_fixture(fixture_root: &Path) -> Result<(), String>
{
    fs::create_dir_all(fixture_root).map_err(|err| {
        format!(
            "failed to create fixture root '{}': {err}",
            fixture_root.display()
        )
    })?;
    fs::create_dir_all(fixture_root.join(".gitvmr")).map_err(|err| {
        format!(
            "failed to create fixture marker '{}': {err}",
            fixture_root.join(".gitvmr").display()
        )
    })?;

    for repo in REPOS
    {
        let repo_dir = fixture_root.join(repo.name);
        if repo_dir.exists()
        {
            reset_default_branch_to_revision(repo, &repo_dir)?;
            continue;
        }

        clone_repo(repo, &repo_dir)?;
        reset_default_branch_to_revision(repo, &repo_dir)?;
    }

    dirty_fixture(fixture_root)?;

    Ok(())
}

fn dirty_fixture(fixture_root: &Path) -> Result<(), String>
{
    edit_file(
        fixture_root,
        "ripgrep",
        "README.md",
        "Temporary benchmark dirt: edited README in the ripgrep fixture.\n\n"
    )?;
    write_untracked_file(
        fixture_root,
        "ripgrep",
        "benchmark-untracked-note.txt",
        "Temporary benchmark dirt: untracked file in the ripgrep fixture.\n"
    )?;
    stage_file(fixture_root, "ripgrep", "benchmark-untracked-note.txt")?;
    move_file(fixture_root, "ripgrep", "FAQ.md", "FAQ.moved.md")?;

    edit_file(
        fixture_root,
        "fd",
        "README.md",
        "Temporary benchmark dirt: edited README in the fd fixture.\n\n"
    )?;
    stage_file(fixture_root, "fd", "README.md")?;
    remove_file(fixture_root, "fd", "SECURITY.md")?;

    edit_file(
        fixture_root,
        "bat",
        "Cargo.toml",
        "# Temporary benchmark dirt: edited Cargo metadata in the bat fixture.\n"
    )?;
    remove_file(fixture_root, "bat", "CHANGELOG.md")?;

    write_untracked_file(
        fixture_root,
        "hyperfine",
        "benchmark-untracked-note.txt",
        "Temporary benchmark dirt: untracked file in the hyperfine fixture.\n"
    )?;
    move_file(
        fixture_root,
        "hyperfine",
        "scripts/README.md",
        "scripts/README.moved.md"
    )
}

fn edit_file(
    fixture_root: &Path,
    repo: &str,
    path: &str,
    prefix: &str
) -> Result<(), String>
{
    let path = fixture_root.join(repo).join(path);
    let content = fs::read_to_string(&path).map_err(|err| {
        format!("failed to read fixture file '{}': {err}", path.display())
    })?;

    fs::write(&path, format!("{prefix}{content}")).map_err(|err| {
        format!("failed to edit fixture file '{}': {err}", path.display())
    })
}

fn write_untracked_file(
    fixture_root: &Path,
    repo: &str,
    path: &str,
    content: &str
) -> Result<(), String>
{
    let path = fixture_root.join(repo).join(path);
    fs::write(&path, content).map_err(|err| {
        format!("failed to write fixture file '{}': {err}", path.display())
    })
}

fn stage_file(fixture_root: &Path, repo: &str, path: &str)
-> Result<(), String>
{
    let repo_dir = fixture_root.join(repo);
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .args(["add", "--", path])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            format!(
                "failed to start staging fixture file '{repo}/{path}': {err}"
            )
        })?;

    if output.status.success()
    {
        Ok(())
    }
    else
    {
        Err(format!(
            "failed to stage fixture file '{repo}/{path}': {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn remove_file(
    fixture_root: &Path,
    repo: &str,
    path: &str
) -> Result<(), String>
{
    let path = fixture_root.join(repo).join(path);
    match fs::remove_file(&path)
    {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!(
            "failed to remove fixture file '{}': {err}",
            path.display()
        ))
    }
}

fn move_file(
    fixture_root: &Path,
    repo: &str,
    from: &str,
    to: &str
) -> Result<(), String>
{
    let repo_dir = fixture_root.join(repo);
    let from = repo_dir.join(from);
    let to = repo_dir.join(to);

    if to.exists()
    {
        fs::remove_file(&to).map_err(|err| {
            format!(
                "failed to remove existing fixture move target '{}': {err}",
                to.display()
            )
        })?;
    }

    fs::rename(&from, &to).map_err(|err| {
        format!(
            "failed to move fixture file '{}' to '{}': {err}",
            from.display(),
            to.display()
        )
    })
}

fn clone_repo(repo: &Repo, repo_dir: &Path) -> Result<(), String>
{
    let output = Command::new("git")
        .args(["clone", repo.url])
        .arg(repo_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            format!("failed to start clone for '{}': {err}", repo.name)
        })?;

    if output.status.success()
    {
        Ok(())
    }
    else
    {
        Err(format!(
            "failed to clone benchmark repository '{}' from '{}' into '{}': {}: {}",
            repo.name,
            repo.url,
            repo_dir.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn reset_default_branch_to_revision(
    repo: &Repo,
    repo_dir: &Path
) -> Result<(), String>
{
    let branch = default_branch(repo, repo_dir)?;
    run_git(
        repo,
        repo_dir,
        &["checkout", branch.as_str()],
        "check out default branch"
    )?;
    run_git(
        repo,
        repo_dir,
        &["reset", "--hard", repo.revision],
        "reset default branch to pinned revision"
    )
}

fn default_branch(repo: &Repo, repo_dir: &Path) -> Result<String, String>
{
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_dir)
        .args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            format!("failed to find default branch for '{}': {err}", repo.name)
        })?;

    if !output.status.success()
    {
        return Err(format!(
            "failed to find default branch for benchmark repository '{}': {}: {}",
            repo.name,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let branch = String::from_utf8(output.stdout).map_err(|err| {
        format!("default branch for '{}' was not UTF-8: {err}", repo.name)
    })?;
    let branch = branch.trim();
    let branch = branch.strip_prefix("origin/").ok_or_else(|| {
        format!(
            "default branch for benchmark repository '{}' was '{branch}', expected origin/<branch>",
            repo.name
        )
    })?;

    Ok(branch.to_owned())
}

fn run_git(
    repo: &Repo,
    repo_dir: &Path,
    args: &[&str],
    action: &str
) -> Result<(), String>
{
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            format!("failed to start {action} for '{}': {err}", repo.name)
        })?;

    if output.status.success()
    {
        return Ok(());
    }

    if action == "reset default branch to pinned revision"
    {
        Err(format!(
            "failed to reset benchmark repository '{}' default branch to revision '{}': {}: {}",
            repo.name,
            repo.revision,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
    else
    {
        Err(format!(
            "failed to {action} for benchmark repository '{}': {}: {}",
            repo.name,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn validate_fixture(fixture_root: &Path) -> Result<(), String>
{
    let marker = fixture_root.join(".gitvmr");
    if !marker.is_dir()
    {
        return Err(format!(
            "fixture root '{}' is missing .gitvmr/",
            fixture_root.display()
        ));
    }

    for repo in REPOS
    {
        let repo_dir = fixture_root.join(repo.name);
        if !repo_dir.is_dir()
        {
            return Err(format!(
                "fixture root '{}' is missing repository directory '{}'",
                fixture_root.display(),
                repo.name
            ));
        }

        if !repo_dir.join(".git").exists()
        {
            return Err(format!(
                "fixture repository '{}' at '{}' is not a git repository",
                repo.name,
                repo_dir.display()
            ));
        }
    }

    Ok(())
}

fn git_vmr_binary() -> Result<PathBuf, String>
{
    if let Some(path) = option_env!("CARGO_BIN_EXE_git-vmr")
    {
        return Ok(PathBuf::from(path));
    }

    let current_exe = env::current_exe().map_err(|err| {
        format!("failed to determine current benchmark executable path: {err}")
    })?;
    for ancestor in current_exe.ancestors()
    {
        if ancestor.file_name().is_some_and(|name| name == "deps")
        {
            let profile_dir = ancestor.parent().ok_or_else(|| {
                format!(
                    "failed to find profile directory from '{}'",
                    current_exe.display()
                )
            })?;
            let candidate =
                profile_dir.join(format!("git-vmr{}", env::consts::EXE_SUFFIX));
            if candidate.is_file()
            {
                return Ok(candidate);
            }
        }
    }

    Err(format!(
        "could not locate compiled git-vmr binary near '{}'",
        current_exe.display()
    ))
}

fn run_status(
    binary: &Path,
    fixture_root: &Path,
    phase: &str
) -> Result<(), String>
{
    let output = Command::new(binary)
        .arg("-C")
        .arg(fixture_root)
        .arg("status")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            format!("failed to start {phase} status command: {err}")
        })?;

    if output.status.success()
    {
        Ok(())
    }
    else
    {
        Err(format!(
            "{phase} status command failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn run_branch(
    binary: &Path,
    fixture_root: &Path,
    phase: &str
) -> Result<(), String>
{
    let output = Command::new(binary)
        .arg("-C")
        .arg(fixture_root)
        .arg("branch")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| {
            format!("failed to start {phase} branch command: {err}")
        })?;

    if output.status.success()
    {
        Ok(())
    }
    else
    {
        Err(format!(
            "{phase} branch command failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1));
    targets = status_real_repos, branch_real_repos
}
criterion_main!(benches);
