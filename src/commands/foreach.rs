use crate::vmr::{Repo, Vmr};
use anyhow::{Result, bail};
use rayon::prelude::*;
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
    working_dir: &Path,
    quiet: bool,
    command: &[String]
) -> Result<()>
{
    let vmr = Vmr::find(working_dir)?;
    let repos = vmr.repos()?;
    let command = command.join(" ");

    let results = repos
        .par_iter()
        .map(|repo| run_child(repo, &vmr, working_dir, &command))
        .collect::<Result<Vec<_>>>()?;

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
    vmr: &Vmr,
    working_dir: &Path,
    command: &str
) -> Result<ChildResult>
{
    let sm_path = repo.path.strip_prefix(&vmr.path).unwrap_or(&repo.path);
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
        .env("toplevel", &vmr.path)
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
