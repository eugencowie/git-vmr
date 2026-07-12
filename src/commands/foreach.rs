use crate::cli::CliContext;
use crate::render::{self, ChildOutput, Rendered};
use crate::workspace::{Repo, Workspace};
use anyhow::Result;
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

/// Evaluates an arbitrary shell command in each checked out repository
#[derive(clap::Args)]
pub struct ForeachArgs
{
    /// Only print error messages
    #[arg(short, long)]
    pub quiet: bool,

    /// Command to evaluate through the shell
    #[arg(
        required = true,
        num_args = 1..,
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "command"
    )]
    pub command: Vec<String>
}

pub fn run(
    workspace: &Workspace,
    context: &CliContext,
    args: ForeachArgs
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    let quiet = args.quiet;
    let command = args.command.join(" ");
    let root = workspace.root();

    let results = workspace
        .map(|_git, repo| run_child(repo, root, working_dir, &command))?;

    let rendered = render::foreach(
        &results
            .iter()
            .map(|result| ChildOutput {
                repo: &result.repo.name,
                stdout: &result.stdout,
                stderr: &result.stderr
            })
            .collect::<Vec<_>>(),
        quiet
    );

    let mut failures = results
        .iter()
        .filter(|result| !result.status.success())
        .map(|result| {
            format!(
                "fatal: command failed in '{}' with {}",
                result.repo.name,
                result.status.describe()
            )
        })
        .collect::<Vec<_>>();
    if !failures.is_empty()
    {
        failures.push("fatal: foreach failed".to_owned());
        return Err(render::fail(rendered, failures.join("\n")));
    }

    Ok(rendered)
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
