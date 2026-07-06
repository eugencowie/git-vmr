use crate::git::GitOutput;
use anyhow::{Context, Result};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

/// The seam through which every git invocation passes. Two adapters satisfy
/// it: [`SubprocessRunner`] in production and the scripted fake in tests.
pub trait GitRunner: Send + Sync
{
    /// Runs git in a repo, capturing its output.
    fn run_captured(
        &self,
        repo_path: &Path,
        args: &[OsString]
    ) -> Result<GitOutput>;

    /// Runs git with inherited stdio, for operations that prompt the user or
    /// stream progress. Git reports its own errors on stderr in this mode.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no caller until clone runs through the seam"
        )
    )]
    fn run_interactive(
        &self,
        working_dir: &Path,
        args: &[OsString]
    ) -> Result<ExitStatus>;
}

pub struct SubprocessRunner;

impl GitRunner for SubprocessRunner
{
    fn run_captured(
        &self,
        repo_path: &Path,
        args: &[OsString]
    ) -> Result<GitOutput>
    {
        let output = Command::new("git")
            .arg("--no-optional-locks")
            .arg("-C")
            .arg(repo_path)
            .args(args)
            .output()
            .with_context(|| {
                format!(
                    "fatal: failed to invoke git for '{}'",
                    repo_path.display()
                )
            })?;

        Ok(GitOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr
        })
    }

    fn run_interactive(
        &self,
        working_dir: &Path,
        args: &[OsString]
    ) -> Result<ExitStatus>
    {
        Command::new("git")
            .arg("-C")
            .arg(working_dir)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| {
                format!(
                    "fatal: failed to invoke git for '{}'",
                    working_dir.display()
                )
            })
    }
}

#[cfg(test)]
pub(crate) mod scripted
{
    use super::*;
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// A recorded invocation: the repo path and args the caller passed.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) struct Invocation
    {
        pub path: PathBuf,
        pub args: Vec<OsString>
    }

    struct Rule
    {
        args: Vec<OsString>,
        status: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>
    }

    /// The scripted fake: answers each expected invocation with a canned
    /// exit code and output, panics on an unexpected one, and records what
    /// was run.
    #[derive(Default)]
    pub(crate) struct ScriptedFake
    {
        captured: Mutex<Vec<Rule>>,
        interactive: Mutex<Vec<Rule>>,
        calls: Mutex<Vec<Invocation>>
    }

    impl ScriptedFake
    {
        pub fn new() -> Self
        {
            Self::default()
        }

        /// Scripts a captured invocation: when git is run with exactly
        /// `args`, answer with `status`, `stdout` and `stderr`.
        pub fn on<S: AsRef<OsStr>>(
            self,
            args: impl IntoIterator<Item = S>,
            status: i32,
            stdout: &str,
            stderr: &str
        ) -> Self
        {
            self.captured.lock().unwrap().push(Rule {
                args: to_args(args),
                status,
                stdout: stdout.as_bytes().to_vec(),
                stderr: stderr.as_bytes().to_vec()
            });
            self
        }

        /// Scripts an interactive invocation, which only has an exit status.
        pub fn on_interactive<S: AsRef<OsStr>>(
            self,
            args: impl IntoIterator<Item = S>,
            status: i32
        ) -> Self
        {
            self.interactive.lock().unwrap().push(Rule {
                args: to_args(args),
                status,
                stdout: Vec::new(),
                stderr: Vec::new()
            });
            self
        }

        /// Every invocation the fake has answered, in call order.
        pub fn calls(&self) -> Vec<Invocation>
        {
            self.calls.lock().unwrap().clone()
        }

        fn answer(
            &self,
            rules: &Mutex<Vec<Rule>>,
            kind: &str,
            path: &Path,
            args: &[OsString]
        ) -> (i32, Vec<u8>, Vec<u8>)
        {
            self.calls.lock().unwrap().push(Invocation {
                path: path.to_owned(),
                args: args.to_vec()
            });

            let rules = rules.lock().unwrap();
            let rule = rules
                .iter()
                .find(|rule| rule.args == args)
                .unwrap_or_else(|| {
                    panic!(
                        "unexpected {kind} git invocation in '{}': git {}",
                        path.display(),
                        args.iter()
                            .map(|arg| arg.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                });
            (rule.status, rule.stdout.clone(), rule.stderr.clone())
        }
    }

    impl GitRunner for ScriptedFake
    {
        fn run_captured(
            &self,
            repo_path: &Path,
            args: &[OsString]
        ) -> Result<GitOutput>
        {
            let (status, stdout, stderr) =
                self.answer(&self.captured, "captured", repo_path, args);
            Ok(GitOutput { status: exit_status(status), stdout, stderr })
        }

        fn run_interactive(
            &self,
            working_dir: &Path,
            args: &[OsString]
        ) -> Result<ExitStatus>
        {
            let (status, ..) = self.answer(
                &self.interactive,
                "interactive",
                working_dir,
                args
            );
            Ok(exit_status(status))
        }
    }

    fn to_args<S: AsRef<OsStr>>(
        args: impl IntoIterator<Item = S>
    ) -> Vec<OsString>
    {
        args.into_iter().map(|arg| arg.as_ref().to_owned()).collect()
    }

    /// Fabricates an [`ExitStatus`] carrying the given exit code.
    pub(crate) fn exit_status(code: i32) -> ExitStatus
    {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(code << 8)
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(code as u32)
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::scripted::ScriptedFake;
    use super::*;
    use std::path::PathBuf;

    fn args<const N: usize>(args: [&str; N]) -> Vec<OsString>
    {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn subprocess_runner_captures_git_output()
    {
        // Act
        let output = SubprocessRunner
            .run_captured(Path::new("."), &args(["--version"]))
            .unwrap();

        // Assert
        assert!(output.status.success());
        assert!(output.stdout.starts_with(b"git version"));
    }

    #[test]
    fn scripted_fake_answers_matching_invocation()
    {
        // Arrange
        let fake = ScriptedFake::new().on(
            ["commit", "-m", "message"],
            0,
            "[main abc1234] message\n",
            ""
        );

        // Act
        let output = fake
            .run_captured(
                Path::new("/vmr/backend"),
                &args(["commit", "-m", "message"])
            )
            .unwrap();

        // Assert
        assert!(output.status.success());
        assert_eq!(output.stdout, b"[main abc1234] message\n");
    }

    #[test]
    fn scripted_fake_reports_scripted_failure_status()
    {
        // Arrange
        let fake =
            ScriptedFake::new().on(["merge", "topic"], 1, "", "merge failed\n");

        // Act
        let output = fake
            .run_captured(Path::new("/vmr/backend"), &args(["merge", "topic"]))
            .unwrap();

        // Assert
        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, b"merge failed\n");
    }

    #[test]
    fn scripted_fake_records_invocations_in_call_order()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on(["fetch"], 0, "", "")
            .on_interactive(["clone", "url"], 0);

        // Act
        fake.run_captured(Path::new("/vmr/backend"), &args(["fetch"])).unwrap();
        fake.run_interactive(Path::new("/vmr"), &args(["clone", "url"]))
            .unwrap();

        // Assert
        let calls = fake.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].path, PathBuf::from("/vmr/backend"));
        assert_eq!(calls[0].args, args(["fetch"]));
        assert_eq!(calls[1].path, PathBuf::from("/vmr"));
        assert_eq!(calls[1].args, args(["clone", "url"]));
    }

    #[test]
    #[should_panic(expected = "unexpected captured git invocation")]
    fn scripted_fake_panics_on_unexpected_invocation()
    {
        // Arrange
        let fake = ScriptedFake::new();

        // Act
        let _ = fake.run_captured(Path::new("/vmr/backend"), &args(["status"]));
    }
}
