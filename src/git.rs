mod add;
mod branch;
mod clone;
mod commit;
mod diff;
mod fetch;
mod merge;
mod mv;
mod pull;
mod push;
mod rebase;
mod reset;
mod restore;
mod rm;
mod status;
mod switch;
mod tag;
mod worktree;

pub use add::{add, add_path};
use anyhow::{Context, Result, bail};
pub use branch::{branch, branches, delete_branch};
pub use clone::clone;
pub use commit::commit;
pub use diff::is_dirty;
pub use fetch::fetch;
pub use merge::merge;
pub use mv::{ensure_tracked, mv};
pub use pull::pull;
pub use push::push;
pub use rebase::rebase;
pub use reset::{ResetMode, reset};
pub use restore::restore;
pub use rm::rm;
pub use status::status;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
pub use switch::{create, switch};
pub use tag::{delete_tag, tag, tags};
pub use worktree::{worktree_add, worktree_move, worktree_remove};

#[derive(Clone, PartialEq, Eq)]
pub enum Head
{
    Branch(String),
    Detached(String)
}

#[derive(Clone, PartialEq, Eq)]
pub struct RepoBranches
{
    pub branches: Vec<String>,
    pub head: Head
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileChange
{
    NewFile,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
    Copied
}

#[derive(Clone, PartialEq, Eq)]
pub struct FileEntry
{
    pub path: PathBuf,
    pub change: FileChange
}

#[derive(Clone, PartialEq, Eq)]
pub struct RepoStatus
{
    pub head: Head,
    pub initial: bool,
    pub staged_changes: Vec<FileEntry>,
    pub unstaged_changes: Vec<FileEntry>,
    pub untracked_files: Vec<FileEntry>
}

pub struct GitOutput
{
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>
}

pub struct RepoMessage
{
    pub repo: String,
    pub message: String
}

pub enum RepoOutcome
{
    Success(Option<RepoMessage>),
    Failure(RepoMessage)
}

pub type GitCommandResult = Result<RepoOutcome>;

pub fn print_results(results: Vec<GitCommandResult>) -> Result<()>
{
    let mut successes = Vec::new();
    let mut failures = Vec::new();
    let mut errors = Vec::new();

    for result in results
    {
        match result
        {
            Ok(RepoOutcome::Success(Some(message))) => successes.push(message),
            Ok(RepoOutcome::Success(None)) =>
            {}
            Ok(RepoOutcome::Failure(message)) => failures.push(message),
            Err(error) => errors.push(error)
        }
    }

    for message in
        grouped_messages(successes, SuccessRepositoryFormat::NamesUntilLimit)
    {
        println!("{message}");
    }

    let mut rendered_errors =
        grouped_messages(failures, SuccessRepositoryFormat::NamesWithCount);
    rendered_errors.extend(errors.into_iter().map(|error| error.to_string()));

    if !rendered_errors.is_empty()
    {
        bail!(rendered_errors.join("\n"));
    }

    Ok(())
}

const REPOSITORY_NAME_LIMIT: usize = 5;

enum SuccessRepositoryFormat
{
    NamesUntilLimit,
    NamesWithCount
}

fn grouped_messages(
    messages: Vec<RepoMessage>,
    format: SuccessRepositoryFormat
) -> Vec<String>
{
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();

    for repo_message in messages
    {
        if let Some((_, repos)) = groups
            .iter_mut()
            .find(|(message, _)| message == &repo_message.message)
        {
            repos.push(repo_message.repo);
        }
        else
        {
            groups.push((repo_message.message, vec![repo_message.repo]));
        }
    }

    groups
        .into_iter()
        .map(|(message, repos)| {
            format!("{message} {}", repository_suffix(&repos, &format))
        })
        .collect()
}

fn repository_suffix(
    repos: &[String],
    format: &SuccessRepositoryFormat
) -> String
{
    if repos.len() == 1
    {
        return format!("({})", repos[0]);
    }

    match format
    {
        SuccessRepositoryFormat::NamesUntilLimit
            if repos.len() > REPOSITORY_NAME_LIMIT =>
        {
            format!("({} repos)", repos.len())
        }
        SuccessRepositoryFormat::NamesUntilLimit =>
            format!("({})", repos.join(", ")),
        SuccessRepositoryFormat::NamesWithCount
            if repos.len() > REPOSITORY_NAME_LIMIT =>
        {
            format!(
                "({} repos: {}, ...)",
                repos.len(),
                repos[..REPOSITORY_NAME_LIMIT].join(", ")
            )
        }
        SuccessRepositoryFormat::NamesWithCount =>
            format!("({})", repos.join(", ")),
    }
}

pub(crate) fn quiet_success() -> RepoOutcome
{
    RepoOutcome::Success(None)
}

pub(crate) fn success_message(repo_name: &str, message: String) -> RepoOutcome
{
    RepoOutcome::Success(Some(RepoMessage {
        repo: repo_name.to_owned(),
        message
    }))
}

pub(crate) fn failure_message(repo_name: &str, message: String) -> RepoOutcome
{
    RepoOutcome::Failure(RepoMessage { repo: repo_name.to_owned(), message })
}

pub(crate) fn command_result(
    repo_name: &str,
    output: &GitOutput,
    success_message: impl FnOnce(&GitOutput) -> Option<String>,
    failure_message: impl FnOnce(&GitOutput) -> String
) -> GitCommandResult
{
    if output.status.success()
    {
        Ok(match success_message(output)
        {
            Some(message) => crate::git::success_message(repo_name, message),
            None => quiet_success()
        })
    }
    else
    {
        Ok(crate::git::failure_message(repo_name, failure_message(output)))
    }
}

pub fn git_output<I, S>(repo_path: &Path, args: I) -> Result<GitOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    let args =
        args.into_iter().map(|arg| arg.as_ref().to_owned()).collect::<Vec<_>>();

    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(repo_path)
        .args(&args)
        .output()
        .with_context(|| {
            format!("fatal: failed to invoke git for '{}'", repo_path.display())
        })?;

    Ok(GitOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr
    })
}

pub fn git_stdout<I, S>(repo_path: &Path, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    let args =
        args.into_iter().map(|arg| arg.as_ref().to_owned()).collect::<Vec<_>>();
    let args_display = format_git_args(&args);
    let output = git_output(repo_path, args)?;

    if !output.status.success()
    {
        bail!(
            "git {} failed for '{}': {}",
            args_display,
            repo_path.display(),
            stderr(&output)
        );
    }

    Ok(output.stdout)
}

pub(crate) fn git_path_output<I, S, P>(
    repo_path: &Path,
    args: I,
    paths: P
) -> Result<GitOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    P: IntoIterator,
    P::Item: AsRef<Path>
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .chain(
            paths.into_iter().map(|path| path.as_ref().as_os_str().to_owned())
        )
        .collect::<Vec<_>>();
    git_output(repo_path, args)
}

pub(crate) fn stderr(output: &GitOutput) -> String
{
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

fn format_git_args(args: &[OsString]) -> String
{
    args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ")
}

pub(crate) fn first_non_empty_line(bytes: &[u8], fallback: &str) -> String
{
    let text = String::from_utf8_lossy(bytes);
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(fallback)
        .to_owned()
}

pub(crate) fn first_non_empty_line_with_fallback(
    primary: &[u8],
    secondary: &[u8],
    fallback: &str
) -> String
{
    let primary_line = first_non_empty_line(primary, "");

    if primary_line.is_empty()
    {
        first_non_empty_line(secondary, fallback)
    }
    else
    {
        primary_line
    }
}

pub(crate) fn status_head(
    repo_path: &Path,
    context: &str,
    header: &[u8]
) -> Result<(Head, bool)>
{
    // Decode and validate branch header.
    let header = String::from_utf8_lossy(header);
    let header = header
        .strip_prefix("## ")
        .context("fatal: git status branch header had unexpected format")?;

    if let Some(branch) = header.strip_prefix("No commits yet on ")
    {
        return Ok((Head::Branch(branch.to_owned()), true));
    }

    if header == "HEAD (no branch)" || header.starts_with("HEAD detached")
    {
        let hash = String::from_utf8_lossy(
            &git_stdout(repo_path, ["rev-parse", "--short", "HEAD"])
                .with_context(|| {
                    format!("fatal: {context} for '{}'", repo_path.display())
                })?
        )
        .trim()
        .to_owned();
        return Ok((Head::Detached(hash), false));
    }

    let branch = header.split("...").next().unwrap_or(header).to_owned();
    Ok((Head::Branch(branch), false))
}

#[cfg(test)]
mod tests
{
    use super::*;
    use anyhow::anyhow;

    fn repo_message(repo: &str, message: &str) -> RepoMessage
    {
        RepoMessage { repo: repo.to_owned(), message: message.to_owned() }
    }

    #[test]
    fn grouped_messages_combines_repositories_with_same_message()
    {
        // Arrange
        let messages = vec![
            repo_message("backend", "Already up to date."),
            repo_message("frontend", "Already up to date."),
            repo_message("tools", "Updating abc123..def456"),
        ];

        // Act
        let rendered = grouped_messages(
            messages,
            SuccessRepositoryFormat::NamesUntilLimit
        );

        // Assert
        assert_eq!(rendered, vec![
            "Already up to date. (backend, frontend)",
            "Updating abc123..def456 (tools)"
        ]);
    }

    #[test]
    fn grouped_success_messages_use_count_for_large_repository_sets()
    {
        // Arrange
        let messages = (1..=6)
            .map(|index| {
                repo_message(&format!("repo-{index}"), "Already up to date.")
            })
            .collect();

        // Act
        let rendered = grouped_messages(
            messages,
            SuccessRepositoryFormat::NamesUntilLimit
        );

        // Assert
        assert_eq!(rendered, vec!["Already up to date. (6 repos)"]);
    }

    #[test]
    fn grouped_failure_messages_keep_repository_sample_for_large_sets()
    {
        // Arrange
        let messages = (1..=6)
            .map(|index| {
                repo_message(&format!("repo-{index}"), "remote rejected")
            })
            .collect();

        // Act
        let rendered =
            grouped_messages(messages, SuccessRepositoryFormat::NamesWithCount);

        // Assert
        assert_eq!(rendered, vec![
            "remote rejected (6 repos: repo-1, repo-2, repo-3, repo-4, repo-5, ...)"
        ]);
    }

    #[test]
    fn print_results_succeeds_for_empty_or_quiet_success_results()
    {
        // Act
        let result = print_results(vec![Ok(RepoOutcome::Success(None))]);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn print_results_groups_failures_and_appends_errors()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Failure(repo_message(
                "backend",
                "error: branch not found"
            ))),
            Ok(RepoOutcome::Failure(repo_message(
                "frontend",
                "error: branch not found"
            ))),
            Err(anyhow!("fatal: transport failed")),
        ];

        // Act
        let err = print_results(results).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: branch not found (backend, frontend)\nfatal: transport failed"
        );
    }
}
