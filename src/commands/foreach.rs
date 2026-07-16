use crate::cli::CliContext;
use crate::render::{Rendered, fail};
use crate::workspace::{Repo, Workspace};
use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};

/// One child repo's captured command output, ready for rendering.
struct ChildOutput<'a>
{
    repo: &'a str,
    stdout: &'a [u8],
    stderr: &'a [u8]
}

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
    args: &ForeachArgs
) -> Result<Rendered>
{
    let working_dir = &context.working_dir;

    let quiet = args.quiet;
    let command = args.command.join(" ");
    let root = workspace.root();

    let results = workspace
        .map(|_git, repo| run_child(repo, root, working_dir, &command))?;

    let rendered = render(
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
        return Err(fail(rendered, failures.join("\n")));
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

/// Renders foreach results: per-repo chrome and child stdout in repo order,
/// then every child's stderr replayed in repo order.
fn render(children: &[ChildOutput], quiet: bool) -> Rendered
{
    let mut stdout = String::new();
    let mut stderr = String::new();

    for child in children
    {
        if !quiet
        {
            stdout.push_str(&format!("Entering '{}'\n", child.repo));
        }
        stdout.push_str(&String::from_utf8_lossy(child.stdout));
    }

    for child in children
    {
        stderr.push_str(&String::from_utf8_lossy(child.stderr));
    }

    Rendered { stdout, stderr }
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

#[cfg(test)]
mod render_tests
{
    use super::*;

    #[test]
    fn renders_chrome_and_stdout_in_repo_order_then_stderr()
    {
        // Arrange
        let children = vec![
            ChildOutput {
                repo: "backend",
                stdout: b"built\n",
                stderr: b"warning: slow\n"
            },
            ChildOutput { repo: "frontend", stdout: b"ok\n", stderr: b"" },
        ];

        // Act
        let rendered = render(&children, false);

        // Assert
        assert_eq!(
            rendered.stdout,
            "Entering 'backend'\nbuilt\nEntering 'frontend'\nok\n"
        );
        assert_eq!(rendered.stderr, "warning: slow\n");
    }

    #[test]
    fn quiet_suppresses_chrome_but_not_child_output()
    {
        // Arrange
        let children = vec![ChildOutput {
            repo: "backend",
            stdout: b"built\n",
            stderr: b""
        }];

        // Act
        let rendered = render(&children, true);

        // Assert
        assert_eq!(rendered.stdout, "built\n");
    }
}

#[cfg(test)]
mod parse_tests
{
    use super::ForeachArgs;
    use crate::cli::Cli;
    use crate::commands::{Command, WorkspaceCommand};
    use clap::error::ErrorKind;

    #[test]
    fn rejects_foreach_without_command()
    {
        // Act
        let err = Cli::parse_from(["git-vmr", "foreach"]).err().unwrap();

        // Assert
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn parses_foreach_quiet_mode()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "foreach", "--quiet", "echo", "ok"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Foreach(ForeachArgs {
                quiet: true,
                command
            })) if command == ["echo", "ok"]
        ));
    }

    #[test]
    fn parses_foreach_multi_word_command()
    {
        // Act
        let cli =
            Cli::parse_from(["git-vmr", "foreach", "git", "status", "--short"])
                .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Foreach(ForeachArgs {
                quiet: false,
                command
            })) if command == ["git", "status", "--short"]
        ));
    }

    #[test]
    fn captures_foreach_child_command_options()
    {
        // Act
        let cli = Cli::parse_from([
            "git-vmr",
            "foreach",
            "echo",
            "--not-a-vmr-option"
        ])
        .unwrap();

        // Assert
        assert!(matches!(
            cli.command,
            Command::Workspace(WorkspaceCommand::Foreach(ForeachArgs {
                quiet: false,
                command
            })) if command == ["echo", "--not-a-vmr-option"]
        ));
    }
}
