use crate::workspace::{Repo, Workspace};
use anyhow::{Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

struct ChildResult
{
    repo: Repo,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: ChildStatus
}

enum ChildStatus
{
    Success,
    Exit(i32),
    Signal
}

impl ChildStatus
{
    fn success(&self) -> bool
    {
        matches!(self, ChildStatus::Success)
    }

    fn describe(&self) -> String
    {
        match self
        {
            ChildStatus::Success => "exit status 0".to_owned(),
            ChildStatus::Exit(code) => format!("exit status {code}"),
            ChildStatus::Signal => "terminated by signal".to_owned()
        }
    }
}

pub fn foreach(
    workspace: &Workspace,
    working_dir: &Path,
    quiet: bool,
    command: &[String]
) -> Result<()>
{
    let command = command.join(" ");
    let root = workspace.root();

    let results = workspace
        .map(|_git, repo| run_child(repo, root, working_dir, &command))?;

    render_results(&results, quiet);

    let failures = results
        .iter()
        .filter(|result| !result.status.success())
        .collect::<Vec<_>>();
    if !failures.is_empty()
    {
        for result in failures
        {
            eprintln!(
                "fatal: command failed in '{}' with {}",
                result.repo.name,
                result.status.describe()
            );
        }
        bail!("fatal: foreach failed");
    }

    Ok(())
}

fn run_child(
    repo: &Repo,
    vmr_root: &Path,
    working_dir: &Path,
    command: &str
) -> Result<ChildResult>
{
    let sm_path = repo.path.strip_prefix(vmr_root).unwrap_or(&repo.path);
    let mut displaypath = pathdiff::diff_paths(&repo.path, working_dir)
        .unwrap_or(repo.path.clone());
    if displaypath.as_os_str().is_empty()
    {
        displaypath.push(".");
    }

    let output = shell_command(command)
        .current_dir(&repo.path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("name", &repo.name)
        .env("sm_path", sm_path)
        .env("displaypath", displaypath)
        .env("toplevel", vmr_root)
        .output()?;

    let status = if output.status.success()
    {
        ChildStatus::Success
    }
    else if let Some(code) = output.status.code()
    {
        ChildStatus::Exit(code)
    }
    else
    {
        ChildStatus::Signal
    };

    Ok(ChildResult {
        repo: repo.clone(),
        stdout: output.stdout,
        stderr: output.stderr,
        status
    })
}

fn shell_command(command: &str) -> Command
{
    #[cfg(windows)]
    {
        let mut child = Command::new("cmd");
        child.arg("/C").arg(command);
        child
    }

    #[cfg(not(windows))]
    {
        let mut child = Command::new("sh");
        child.arg("-c").arg(command);
        child
    }
}

fn render_results(results: &[ChildResult], quiet: bool)
{
    for result in results
    {
        if !quiet
        {
            println!("Entering '{}'", result.repo.name);
        }
        print!("{}", String::from_utf8_lossy(&result.stdout));
    }

    for result in results
    {
        eprint!("{}", String::from_utf8_lossy(&result.stderr));
    }
}
