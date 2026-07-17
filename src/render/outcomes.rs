use crate::git::{GitCommandResult, RepoMessage, RepoOutcome};
use crate::render::{
    Rendered, SKIPPED, SuffixPolicy, fail, paint, repo_list_suffix
};
use anstyle::Style;
use anyhow::Result;
use std::collections::HashSet;

/// Renders aggregated per-repo outcomes against the child repos in scope:
/// identical messages are grouped, successes become stdout, and failures
/// become the command's error. The suffix is omitted when a group covers
/// every repo in scope.
///
/// The scope — repo names, duplicates allowed — must be supplied by the
/// caller: outcomes cannot self-describe it, because quiet successes and
/// hard errors carry no repo name, and a repo in scope (a move's
/// destination) may produce no outcome at all.
pub fn outcomes<'a>(
    results: Vec<GitCommandResult>,
    scope: impl IntoIterator<Item = &'a str>
) -> Result<Rendered>
{
    let total = scope.into_iter().collect::<HashSet<_>>().len();
    let mut successes = Vec::new();
    let mut skips = Vec::new();
    let mut failures = Vec::new();
    let mut errors = Vec::new();

    for result in results
    {
        match result
        {
            Ok(RepoOutcome::Success(Some(message))) => successes.push(message),
            Ok(RepoOutcome::Success(None)) =>
            {}
            Ok(RepoOutcome::Skipped(message)) => skips.push(message),
            Ok(RepoOutcome::Failure(message)) => failures.push(message),
            Err(error) => errors.push(error)
        }
    }

    let mut stdout = String::new();
    for message in grouped_messages(successes, total, SuffixPolicy::Truncated)
        .into_iter()
        .chain(grouped_styled_messages(
            skips,
            total,
            SuffixPolicy::Truncated,
            SKIPPED
        ))
    {
        stdout.push_str(&message);
        stdout.push('\n');
    }

    let rendered = Rendered { stdout, stderr: String::new() };

    let mut rendered_errors =
        grouped_messages(failures, total, SuffixPolicy::Full);
    rendered_errors
        .extend(errors.into_iter().map(|error| format!("{error:#}")));

    if !rendered_errors.is_empty()
    {
        return Err(fail(rendered, rendered_errors.join("\n")));
    }

    Ok(rendered)
}

fn grouped_messages(
    messages: Vec<RepoMessage>,
    total: usize,
    policy: SuffixPolicy
) -> Vec<String>
{
    grouped_lines(messages, total, policy)
        .map(|(message, suffix)| format!("{message}{suffix}"))
        .collect()
}

/// Like [`grouped_messages`], but paints each group's message with `style`
/// after grouping — grouping always compares raw messages, never styled text.
fn grouped_styled_messages(
    messages: Vec<RepoMessage>,
    total: usize,
    policy: SuffixPolicy,
    style: Style
) -> Vec<String>
{
    grouped_lines(messages, total, policy)
        .map(|(message, suffix)| format!("{}{suffix}", paint(style, &message)))
        .collect()
}

/// Groups identical messages and pairs each with its repo-list suffix.
fn grouped_lines(
    messages: Vec<RepoMessage>,
    total: usize,
    policy: SuffixPolicy
) -> impl Iterator<Item = (String, String)>
{
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();

    for repo_message in messages
    {
        if let Some((_, repos)) = groups
            .iter_mut()
            .find(|(message, _)| message == &repo_message.message)
        {
            if !repos.contains(&repo_message.repo)
            {
                repos.push(repo_message.repo);
            }
        }
        else
        {
            groups.push((repo_message.message, vec![repo_message.repo]));
        }
    }

    groups.into_iter().map(move |(message, repos)| {
        let repos = repos.iter().map(String::as_str).collect::<Vec<_>>();
        let suffix = repo_list_suffix(&repos, total, policy);
        (message, suffix)
    })
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
        let rendered = grouped_messages(messages, 3, SuffixPolicy::Truncated);

        // Assert
        assert_eq!(rendered, vec![
            "Already up to date. \x1b[90m(backend, frontend)\x1b[0m",
            "Updating abc123..def456 \x1b[90m(tools)\x1b[0m"
        ]);
    }

    #[test]
    fn grouped_messages_omit_the_repo_list_when_a_group_covers_all_repos()
    {
        // Arrange
        let messages = vec![
            repo_message("backend", "Already up to date."),
            repo_message("frontend", "Already up to date."),
        ];

        // Act
        let rendered = grouped_messages(messages, 2, SuffixPolicy::Truncated);

        // Assert
        assert_eq!(rendered, vec!["Already up to date."]);
    }

    #[test]
    fn grouped_success_messages_truncate_large_repository_sets()
    {
        // Arrange
        let messages = (1..=6)
            .map(|index| {
                repo_message(&format!("repo-{index}"), "Already up to date.")
            })
            .collect();

        // Act
        let rendered = grouped_messages(messages, 7, SuffixPolicy::Truncated);

        // Assert
        assert_eq!(rendered, vec![
            "Already up to date. \x1b[90m(repo-1, repo-2, repo-3, +3)\x1b[0m"
        ]);
    }

    #[test]
    fn grouped_failure_messages_name_every_repository()
    {
        // Arrange
        let messages = (1..=6)
            .map(|index| {
                repo_message(&format!("repo-{index}"), "remote rejected")
            })
            .collect();

        // Act
        let rendered = grouped_messages(messages, 7, SuffixPolicy::Full);

        // Assert
        assert_eq!(rendered, vec![
            "remote rejected \x1b[90m(repo-1, repo-2, repo-3, repo-4, repo-5, repo-6)\x1b[0m"
        ]);
    }

    #[test]
    fn outcomes_succeeds_for_empty_or_quiet_success_results()
    {
        // Act
        let result =
            outcomes(vec![Ok(RepoOutcome::Success(None))], ["backend"]);

        // Assert
        assert!(result.unwrap().stdout.is_empty());
    }

    #[test]
    fn outcomes_renders_grouped_successes_as_stdout()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Success(Some(repo_message(
                "backend",
                "Already up to date."
            )))),
            Ok(RepoOutcome::Success(Some(repo_message(
                "frontend",
                "Already up to date."
            )))),
        ];

        // Act
        let rendered = outcomes(results, ["backend", "frontend"]).unwrap();

        // Assert
        assert_eq!(rendered.stdout, "Already up to date.\n");
    }

    #[test]
    fn outcomes_keep_the_repo_list_when_quiet_successes_share_the_scope()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Success(Some(repo_message(
                "backend",
                "Already up to date."
            )))),
            Ok(RepoOutcome::Success(None)),
        ];

        // Act
        let rendered = outcomes(results, ["backend", "frontend"]).unwrap();

        // Assert
        assert_eq!(
            rendered.stdout,
            "Already up to date. \x1b[90m(backend)\x1b[0m\n"
        );
    }

    #[test]
    fn outcomes_group_identical_skip_reasons_dimmed_on_stdout()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Skipped(repo_message(
                "frontend",
                "nothing to push"
            ))),
            Ok(RepoOutcome::Skipped(repo_message("docs", "nothing to push"))),
        ];

        // Act
        let rendered =
            outcomes(results, ["backend", "frontend", "docs"]).unwrap();

        // Assert
        assert_eq!(
            rendered.stdout,
            "\x1b[2mnothing to push\x1b[0m \x1b[90m(frontend, docs)\x1b[0m\n"
        );
    }

    #[test]
    fn outcomes_with_skips_do_not_change_the_exit_code()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Success(Some(repo_message("backend", "pushed")))),
            Ok(RepoOutcome::Skipped(repo_message("docs", "nothing to push"))),
        ];

        // Act
        let rendered = outcomes(results, ["backend", "docs"]).unwrap();

        // Assert
        assert_eq!(
            rendered.stdout,
            "pushed \x1b[90m(backend)\x1b[0m\n\x1b[2mnothing to \
             push\x1b[0m \x1b[90m(docs)\x1b[0m\n"
        );
    }

    #[test]
    fn outcomes_all_skipped_succeed_and_are_not_silent()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Skipped(repo_message(
                "backend",
                "nothing to push"
            ))),
            Ok(RepoOutcome::Skipped(repo_message(
                "frontend",
                "nothing to push"
            ))),
        ];

        // Act
        let rendered = outcomes(results, ["backend", "frontend"]).unwrap();

        // Assert: the suffix is omitted (everyone), but the reason prints.
        assert_eq!(rendered.stdout, "\x1b[2mnothing to push\x1b[0m\n");
    }

    #[test]
    fn outcomes_print_skips_when_failures_fail_the_command()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Skipped(repo_message(
                "backend",
                "nothing to push"
            ))),
            Ok(RepoOutcome::Failure(repo_message(
                "frontend",
                "remote rejected"
            ))),
        ];

        // Act
        let err = outcomes(results, ["backend", "frontend"]).unwrap_err();

        // Assert
        let failed = err.downcast::<crate::render::Failed>().unwrap();
        assert_eq!(
            failed.rendered.stdout,
            "\x1b[2mnothing to push\x1b[0m \x1b[90m(backend)\x1b[0m\n"
        );
        assert_eq!(failed.message, "remote rejected \x1b[90m(frontend)\x1b[0m");
    }

    #[test]
    fn outcomes_groups_failures_and_appends_errors()
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
        let err =
            outcomes(results, ["backend", "frontend", "tools"]).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "error: branch not found \x1b[90m(backend, frontend)\x1b[0m\nfatal: transport failed"
        );
    }

    #[test]
    fn outcomes_renders_error_context()
    {
        // Arrange
        let error =
            Err(anyhow!("connection refused")
                .context("failed to fetch repository"));

        // Act
        let err = outcomes(vec![error], ["backend"]).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "failed to fetch repository: connection refused"
        );
    }

    #[test]
    fn outcomes_keeps_partial_successes_on_failure()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Success(Some(repo_message("backend", "pushed")))),
            Ok(RepoOutcome::Failure(repo_message(
                "frontend",
                "remote rejected"
            ))),
        ];

        // Act
        let err = outcomes(results, ["backend", "frontend"]).unwrap_err();

        // Assert
        let failed = err.downcast::<crate::render::Failed>().unwrap();
        assert_eq!(failed.rendered.stdout, "pushed \x1b[90m(backend)\x1b[0m\n");
        assert_eq!(failed.message, "remote rejected \x1b[90m(frontend)\x1b[0m");
    }

    #[test]
    fn scope_deduplicates_repeated_repo_names()
    {
        // Arrange: two move entries failed in the same destination repo,
        // within a command whose scope names that repo once per entry.
        let results = vec![
            Ok(RepoOutcome::Failure(repo_message(
                "frontend",
                "unable to stage"
            ))),
            Ok(RepoOutcome::Failure(repo_message(
                "frontend",
                "unable to stage"
            ))),
        ];

        // Act
        let err =
            outcomes(results, ["backend", "frontend", "frontend"]).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "unable to stage \x1b[90m(frontend)\x1b[0m"
        );
    }

    #[test]
    fn scope_repos_without_outcomes_widen_the_denominator()
    {
        // Arrange: a move's destination repo is in scope but produces no
        // outcome, so a failure confined to the source must keep its suffix.
        let results =
            vec![Ok(RepoOutcome::Failure(repo_message("backend", "fatal")))];

        // Act
        let err = outcomes(results, ["backend", "frontend"]).unwrap_err();

        // Assert
        assert_eq!(err.to_string(), "fatal \x1b[90m(backend)\x1b[0m");
    }

    #[test]
    fn suffix_is_omitted_when_a_group_covers_the_whole_scope()
    {
        // Arrange
        let results = vec![
            Ok(RepoOutcome::Success(Some(repo_message("backend", "pushed")))),
            Ok(RepoOutcome::Success(Some(repo_message("frontend", "pushed")))),
        ];

        // Act
        let rendered =
            outcomes(results, ["backend", "frontend", "backend"]).unwrap();

        // Assert
        assert_eq!(rendered.stdout, "pushed\n");
    }
}
