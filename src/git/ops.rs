//! The simple git operations: each builds args, runs through the seam, and
//! maps output to a repo outcome. Queries with real parsing live in their
//! own modules (status, branch, worktree).

use crate::git::{
    Git, GitCommandResult, command_result, failure_message,
    first_non_empty_line, first_non_empty_line_with_fallback, stderr,
    success_message
};
use anyhow::{Context, Result, bail};
use regex::Regex;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChmodMode
{
    Executable,
    NotExecutable
}

impl ChmodMode
{
    fn as_git_value(self) -> &'static str
    {
        match self
        {
            ChmodMode::Executable => "+x",
            ChmodMode::NotExecutable => "-x"
        }
    }
}

impl Display for ChmodMode
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(self.as_git_value())
    }
}

impl FromStr for ChmodMode
{
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err>
    {
        match value
        {
            "+x" => Ok(ChmodMode::Executable),
            "-x" => Ok(ChmodMode::NotExecutable),
            _ => Err("expected +x or -x".to_owned())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetMode
{
    Soft,
    Mixed,
    Hard,
    Merge,
    Keep
}

impl ResetMode
{
    pub fn from_arg(
        soft: bool,
        mixed: bool,
        hard: bool,
        merge: bool,
        keep: bool
    ) -> Option<ResetMode>
    {
        match (soft, mixed, hard, merge, keep)
        {
            (true, false, false, false, false) => Some(ResetMode::Soft),
            (false, true, false, false, false) => Some(ResetMode::Mixed),
            (false, false, true, false, false) => Some(ResetMode::Hard),
            (false, false, false, true, false) => Some(ResetMode::Merge),
            (false, false, false, false, true) => Some(ResetMode::Keep),
            (false, false, false, false, false) => None,
            _ => unreachable!()
        }
    }

    pub fn as_arg(self) -> &'static str
    {
        match self
        {
            ResetMode::Soft => "--soft",
            ResetMode::Mixed => "--mixed",
            ResetMode::Hard => "--hard",
            ResetMode::Merge => "--merge",
            ResetMode::Keep => "--keep"
        }
    }
}

static BEHIND_COMMIT_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" by \d+ commits?").unwrap());

fn normalize_success_message(message: String) -> String
{
    BEHIND_COMMIT_COUNT.replace(&message, "").into_owned()
}

fn add_args(all: bool, force: bool, chmod: Option<ChmodMode>) -> Vec<OsString>
{
    let mut args = vec![OsString::from("add")];

    if all
    {
        args.push(OsString::from("--all"));
    }

    if force
    {
        args.push(OsString::from("--force"));
    }

    if let Some(chmod) = chmod
    {
        args.push(OsString::from(format!("--chmod={chmod}")));
    }

    args.push(OsString::from("--"));
    args
}

impl Git
{
    pub fn add(
        &self,
        repo_name: &str,
        repo_path: &Path,
        paths: &[PathBuf],
        all: bool,
        force: bool,
        chmod: Option<ChmodMode>
    ) -> GitCommandResult
    {
        let output =
            self.path_output(repo_path, add_args(all, force, chmod), paths)?;

        command_result(
            repo_name,
            &output,
            |_| None,
            |output| {
                format!(
                    "git add failed for '{}': {}",
                    repo_path.display(),
                    stderr(output)
                )
            }
        )
    }

    pub fn add_path(&self, repo_path: &Path, path: &Path) -> Result<()>
    {
        let output = self.path_output(repo_path, ["add", "--"], [path])?;

        if !output.status.success()
        {
            bail!(
                "git add failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }

    pub fn commit(
        &self,
        repo_name: &str,
        repo_path: &Path,
        message: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["commit", "-m", message])?;

        command_result(
            repo_name,
            &output,
            |output| {
                Some(first_non_empty_line(
                    &output.stdout,
                    "git commit succeeded"
                ))
            },
            |output| first_non_empty_line(&output.stderr, "git commit failed")
        )
    }

    pub fn is_dirty(&self, repo_path: &Path) -> Result<bool>
    {
        let output = self
            .output(repo_path, ["diff", "--cached", "--quiet"])
            .with_context(|| {
                format!(
                    "fatal: failed to inspect staged changes for '{}'",
                    repo_path.display()
                )
            })?;

        match output.status.code()
        {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => bail!(
                "fatal: git diff --cached --quiet failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            )
        }
    }

    pub fn fetch(
        &self,
        repo_name: &str,
        repo_path: &Path,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("fetch")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    ""
                );

                if message.is_empty() { None } else { Some(message) }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git fetch failed"
                )
            }
        )
    }

    pub fn merge(
        &self,
        repo_name: &str,
        repo_path: &Path,
        commit_ish: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["merge", commit_ish])?;

        command_result(
            repo_name,
            &output,
            |output| {
                Some(first_non_empty_line(
                    &output.stdout,
                    "git merge succeeded"
                ))
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git merge failed"
                )
            }
        )
    }

    pub fn mv(
        &self,
        repo_path: &Path,
        source: &Path,
        destination: &Path
    ) -> Result<()>
    {
        let output =
            self.path_output(repo_path, ["mv", "--"], [source, destination])?;

        if !output.status.success()
        {
            bail!(
                "git mv failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }

    pub fn mv_to_directory(
        &self,
        repo_path: &Path,
        sources: &[PathBuf],
        destination: &Path
    ) -> Result<()>
    {
        let paths = sources
            .iter()
            .map(PathBuf::as_path)
            .chain(std::iter::once(destination));
        let output = self.path_output(repo_path, ["mv", "--"], paths)?;

        if !output.status.success()
        {
            bail!(
                "git mv failed for '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }

    pub fn ensure_tracked(&self, repo_path: &Path, path: &Path) -> Result<()>
    {
        let output = self.path_output(
            repo_path,
            ["ls-files", "--error-unmatch", "--"],
            [path]
        )?;

        if !output.status.success()
        {
            bail!(
                "source path is not tracked in '{}': {}",
                repo_path.display(),
                stderr(&output)
            );
        }

        Ok(())
    }

    pub fn pull(
        &self,
        repo_name: &str,
        repo_path: &Path,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("pull")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stdout,
                    &output.stderr,
                    ""
                );

                if message.is_empty() { None } else { Some(message) }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git pull failed"
                )
            }
        )
    }

    pub fn push(
        &self,
        repo_name: &str,
        repo_path: &Path,
        repository: Option<&str>,
        refspecs: &[String]
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("push")];

        if let Some(repository) = repository
        {
            args.push(OsString::from(repository));
        }

        args.extend(refspecs.iter().map(OsString::from));

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    ""
                );

                if message.is_empty() { None } else { Some(message) }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git push failed"
                )
            }
        )
    }

    pub fn rebase(
        &self,
        repo_name: &str,
        repo_path: &Path,
        upstream: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["rebase", upstream])?;

        command_result(
            repo_name,
            &output,
            |_| None,
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git rebase failed"
                )
            }
        )
    }

    pub fn reset(
        &self,
        repo_name: &str,
        repo_path: &Path,
        mode: Option<ResetMode>,
        commit: Option<&str>
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("reset")];

        if let Some(mode) = mode
        {
            args.push(OsString::from(mode.as_arg()));
        }

        if let Some(commit) = commit
        {
            args.push(OsString::from(commit));
        }

        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                let message = first_non_empty_line_with_fallback(
                    &output.stdout,
                    &output.stderr,
                    ""
                );

                if message.is_empty() { None } else { Some(message) }
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git reset failed"
                )
            }
        )
    }

    pub fn restore(
        &self,
        repo_name: &str,
        repo_path: &Path,
        paths: &[PathBuf],
        worktree: bool,
        staged: bool
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("restore")];

        if staged
        {
            args.push(OsString::from("--staged"));
        }

        if worktree
        {
            args.push(OsString::from("--worktree"));
        }

        args.push(OsString::from("--"));
        args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |_| None,
            |output| {
                format!(
                    "git restore failed for '{}': {}",
                    repo_path.display(),
                    stderr(output)
                )
            }
        )
    }

    #[expect(clippy::too_many_arguments, reason = "mirrors git rm flags")]
    pub fn rm(
        &self,
        repo_name: &str,
        repo_path: &Path,
        paths: &[PathBuf],
        recursive: bool,
        force: bool,
        dry_run: bool,
        cached: bool
    ) -> GitCommandResult
    {
        let mut args = vec![OsString::from("rm")];

        if recursive
        {
            args.push(OsString::from("-r"));
        }

        if force
        {
            args.push(OsString::from("--force"));
        }

        if dry_run
        {
            args.push(OsString::from("--dry-run"));
        }

        if cached
        {
            args.push(OsString::from("--cached"));
        }

        args.push(OsString::from("--"));
        args.extend(paths.iter().map(|path| path.as_os_str().to_owned()));
        let output = self.output(repo_path, args)?;

        command_result(
            repo_name,
            &output,
            |output| {
                if dry_run
                {
                    let message = first_non_empty_line(&output.stdout, "");

                    if message.is_empty() { None } else { Some(message) }
                }
                else
                {
                    None
                }
            },
            |output| {
                format!(
                    "git rm failed for '{}': {}",
                    repo_path.display(),
                    stderr(output)
                )
            }
        )
    }

    pub fn switch(
        &self,
        repo_name: &str,
        repo_path: &Path,
        branch_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["switch", branch_name])?;

        command_result(
            repo_name,
            &output,
            |output| {
                Some(normalize_success_message(
                    first_non_empty_line_with_fallback(
                        &output.stdout,
                        &output.stderr,
                        "git switch succeeded"
                    )
                ))
            },
            |output| {
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git switch failed"
                )
            }
        )
    }

    pub fn create(
        &self,
        repo_name: &str,
        repo_path: &Path,
        branch_name: &str
    ) -> GitCommandResult
    {
        let output =
            self.output(repo_path, ["switch", "--create", branch_name])?;

        if output.status.success()
        {
            Ok(success_message(
                repo_name,
                first_non_empty_line_with_fallback(
                    &output.stdout,
                    &output.stderr,
                    "git switch succeeded"
                )
            ))
        }
        else
        {
            Ok(failure_message(
                repo_name,
                first_non_empty_line_with_fallback(
                    &output.stderr,
                    &output.stdout,
                    "git switch failed"
                )
            ))
        }
    }

    pub fn tag(
        &self,
        repo_name: &str,
        repo_path: &Path,
        tag_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["tag", tag_name])?;

        command_result(
            repo_name,
            &output,
            |_| None,
            |output| first_non_empty_line(&output.stderr, "git tag failed")
        )
    }

    pub fn delete_tag(
        &self,
        repo_name: &str,
        repo_path: &Path,
        tag_name: &str
    ) -> GitCommandResult
    {
        let output = self.output(repo_path, ["tag", "-d", tag_name])?;

        command_result(
            repo_name,
            &output,
            |output| {
                first_non_empty_line(&output.stdout, "git tag delete succeeded")
                    .into()
            },
            |output| first_non_empty_line(&output.stderr, "git tag failed")
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::RepoOutcome;
    use crate::git::runner::scripted::ScriptedFake;

    #[test]
    fn from_arg_returns_none_when_no_mode_flag_is_set()
    {
        // Act
        let mode = ResetMode::from_arg(false, false, false, false, false);

        // Assert
        assert_eq!(mode, None);
    }

    #[test]
    fn from_arg_returns_selected_mode()
    {
        for (flags, expected) in [
            ((true, false, false, false, false), ResetMode::Soft),
            ((false, true, false, false, false), ResetMode::Mixed),
            ((false, false, true, false, false), ResetMode::Hard),
            ((false, false, false, true, false), ResetMode::Merge),
            ((false, false, false, false, true), ResetMode::Keep)
        ]
        {
            // Act
            let mode = ResetMode::from_arg(
                flags.0, flags.1, flags.2, flags.3, flags.4
            );

            // Assert
            assert_eq!(mode, Some(expected));
        }
    }

    #[test]
    fn normalizes_behind_fast_forward_success_message()
    {
        assert_eq!(
            normalize_success_message(
                "Your branch is behind 'origin/develop' by 1 commit, and can be fast-forwarded."
                    .to_owned()
            ),
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
        assert_eq!(
            normalize_success_message(
                "Your branch is behind 'origin/develop' by 20 commits, and can be fast-forwarded."
                    .to_owned()
            ),
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
    }

    #[test]
    fn leaves_other_success_messages_unchanged()
    {
        assert_eq!(
            normalize_success_message(
                "Switched to branch 'feature/auth'".to_owned()
            ),
            "Switched to branch 'feature/auth'"
        );
    }

    #[test]
    fn fetch_with_no_output_is_a_quiet_success()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(["fetch"], 0, "", ""));

        // Act
        let outcome =
            git.fetch("backend", Path::new("/vmr/backend"), None, &[]).unwrap();

        // Assert
        assert!(matches!(outcome, RepoOutcome::Success(None)));
    }

    #[test]
    fn delete_tag_with_no_output_reports_success()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["tag", "-d", "v1.0.0"],
            0,
            "",
            ""
        ));

        // Act
        let outcome = git
            .delete_tag("backend", Path::new("/vmr/backend"), "v1.0.0")
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "git tag delete succeeded");
    }

    #[test]
    fn is_dirty_maps_diff_exit_codes()
    {
        for (code, expected) in [(0, false), (1, true)]
        {
            // Arrange
            let git = Git::with(ScriptedFake::new().on(
                ["diff", "--cached", "--quiet"],
                code,
                "",
                ""
            ));

            // Act
            let dirty = git.is_dirty(Path::new("/vmr/backend")).unwrap();

            // Assert
            assert_eq!(dirty, expected);
        }
    }

    #[test]
    fn switch_normalizes_behind_count_in_success_message()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["switch", "develop"],
            0,
            "Your branch is behind 'origin/develop' by 3 commits, and can be fast-forwarded.\n",
            ""
        ));

        // Act
        let outcome = git
            .switch("backend", Path::new("/vmr/backend"), "develop")
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(
            message.message,
            "Your branch is behind 'origin/develop', and can be fast-forwarded."
        );
    }

    #[test]
    fn rm_reports_stdout_only_for_dry_runs()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["rm", "--dry-run", "--", "old.rs"],
            0,
            "rm 'old.rs'\n",
            ""
        ));

        // Act
        let outcome = git
            .rm(
                "backend",
                Path::new("/vmr/backend"),
                &[PathBuf::from("old.rs")],
                false,
                false,
                true,
                false
            )
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.message, "rm 'old.rs'");
    }

    #[test]
    fn commit_reports_first_stdout_line_through_the_seam()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["commit", "-m", "message"],
            0,
            "[main abc1234] message\n 1 file changed\n",
            ""
        ));

        // Act
        let outcome = git
            .commit("backend", Path::new("/vmr/backend"), "message")
            .unwrap();

        // Assert
        let RepoOutcome::Success(Some(message)) = outcome
        else
        {
            panic!("expected success with message");
        };
        assert_eq!(message.repo, "backend");
        assert_eq!(message.message, "[main abc1234] message");
    }

    #[test]
    fn commit_reports_first_stderr_line_on_failure()
    {
        // Arrange
        let git = Git::with(ScriptedFake::new().on(
            ["commit", "-m", "message"],
            1,
            "",
            "error: nothing to commit\n"
        ));

        // Act
        let outcome = git
            .commit("backend", Path::new("/vmr/backend"), "message")
            .unwrap();

        // Assert
        let RepoOutcome::Failure(message) = outcome
        else
        {
            panic!("expected failure");
        };
        assert_eq!(message.message, "error: nothing to commit");
    }
}
