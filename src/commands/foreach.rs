use crate::cli::CliContext;
use crate::render::{Rendered, fail};
use crate::workspace::{Repo, RepoKind, Workspace};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

/// One member repo's captured command output, ready for rendering.
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

impl From<ExitStatus> for ChildStatus
{
    fn from(status: ExitStatus) -> Self
    {
        if status.success()
        {
            ChildStatus::Success
        }
        else if let Some(code) = status.code()
        {
            ChildStatus::Exit(code)
        }
        else
        {
            ChildStatus::Signal
        }
    }
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
                repo: &result.repo.path,
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
                result.repo.path,
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

/// The member environment's per-repo variables: the repo's manifest name
/// and path from the VMR root (`.` for both, for the root repo), and its
/// path relative to the working dir.
struct MemberEnvironment<'a>
{
    name: &'a str,
    sm_path: &'a str,
    displaypath: PathBuf
}

fn member_environment<'a>(
    repo: &'a Repo,
    working_dir: &Path
) -> MemberEnvironment<'a>
{
    // Name the repo as the manifest does, and the root repo `.`
    let name = match &repo.kind
    {
        RepoKind::Root => ".",
        RepoKind::Child { name } => name
    };

    // Show the repo relative to the working dir
    let mut displaypath =
        pathdiff::diff_paths(&repo.absolute_path, working_dir)
            .unwrap_or_else(|| repo.absolute_path.clone());
    if displaypath.as_os_str().is_empty()
    {
        displaypath.push(".");
    }

    MemberEnvironment { name, sm_path: &repo.path, displaypath }
}

fn run_child(
    repo: &Repo,
    vmr_root: &Path,
    working_dir: &Path,
    command: &str
) -> Result<ChildResult>
{
    let environment = member_environment(repo, working_dir);

    let output = shell_command(command)
        .current_dir(&repo.absolute_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("name", environment.name)
        .env("sm_path", environment.sm_path)
        .env("displaypath", environment.displaypath)
        .env("toplevel", vmr_root)
        .output()?;

    Ok(ChildResult {
        repo: repo.clone(),
        stdout: output.stdout,
        stderr: output.stderr,
        status: output.status.into()
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

    Rendered { stdout, stderr, pager: None }
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
mod member_environment_tests
{
    use super::*;
    use crate::test_support::repo;

    #[test]
    fn working_dir_at_the_vmr_root_yields_the_repo_path()
    {
        // Arrange
        let backend = repo("backend", "/vmr/backend");

        // Act
        let environment = member_environment(&backend, Path::new("/vmr"));

        // Assert
        assert_eq!(environment.name, "backend");
        assert_eq!(environment.sm_path, "backend");
        assert_eq!(environment.displaypath, Path::new("backend"));
    }

    #[test]
    fn the_root_repo_is_named_and_placed_at_dot()
    {
        // Arrange
        let root = Repo::root(Path::new("/vmr"));

        // Act
        let environment = member_environment(&root, Path::new("/vmr"));

        // Assert
        assert_eq!(environment.name, ".");
        assert_eq!(environment.sm_path, ".");
        assert_eq!(environment.displaypath, Path::new("."));
    }

    #[test]
    fn working_dir_inside_a_sibling_child_traverses_up()
    {
        // Arrange
        let backend = repo("backend", "/vmr/backend");

        // Act
        let environment =
            member_environment(&backend, Path::new("/vmr/frontend"));

        // Assert
        assert_eq!(environment.displaypath, Path::new("../backend"));
    }

    #[test]
    fn working_dir_equal_to_the_repo_yields_dot()
    {
        // Arrange
        let backend = repo("backend", "/vmr/backend");

        // Act
        let environment =
            member_environment(&backend, Path::new("/vmr/backend"));

        // Assert
        assert_eq!(environment.displaypath, Path::new("."));
    }

    #[test]
    fn working_dir_nested_inside_the_repo_traverses_up()
    {
        // Arrange
        let backend = repo("backend", "/vmr/backend");

        // Act
        let environment =
            member_environment(&backend, Path::new("/vmr/backend/src"));

        // Assert
        assert_eq!(environment.displaypath, Path::new(".."));
    }

    #[test]
    fn displaypath_without_a_relative_route_falls_back_to_the_repo_path()
    {
        // Arrange: the repo and working dir have no relative route
        #[cfg(unix)]
        let (repo_path, working_dir, expected) =
            ("rel/backend", "/vmr", Path::new("rel/backend"));
        #[cfg(windows)]
        let (repo_path, working_dir, expected) =
            (r"D:\rel\backend", r"C:\vmr", Path::new(r"D:\rel\backend"));
        let backend = repo("backend", repo_path);

        // Act
        let environment = member_environment(&backend, Path::new(working_dir));

        // Assert
        assert_eq!(environment.displaypath, expected);
    }
}

#[cfg(test)]
mod child_status_tests
{
    use super::*;

    #[cfg(unix)]
    fn exit_status(code: i32) -> ExitStatus
    {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code << 8)
    }

    #[cfg(windows)]
    fn exit_status(code: i32) -> ExitStatus
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }

    #[test]
    fn zero_exit_converts_to_success()
    {
        // Act
        let status = ChildStatus::from(exit_status(0));

        // Assert
        assert!(status.success());
        assert_eq!(status.describe(), "exit status 0");
    }

    #[test]
    fn nonzero_exit_converts_to_exit_with_its_code()
    {
        // Act
        let status = ChildStatus::from(exit_status(3));

        // Assert
        assert!(!status.success());
        assert_eq!(status.describe(), "exit status 3");
    }

    #[cfg(unix)]
    #[test]
    fn signal_termination_converts_to_signal()
    {
        use std::os::unix::process::ExitStatusExt;

        // Act
        let status = ChildStatus::from(ExitStatus::from_raw(9));

        // Assert
        assert!(!status.success());
        assert_eq!(status.describe(), "terminated by signal");
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
