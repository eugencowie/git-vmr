use crate::git::{GitCommandResult, RepoMessage, RepoOutcome};
use crate::render::{Rendered, SuffixPolicy, fail, repo_list_suffix};
use anyhow::Result;

/// Renders aggregated per-repo outcomes: identical messages are grouped,
/// successes become stdout, and failures become the command's error.
pub fn outcomes(results: Vec<GitCommandResult>) -> Result<Rendered>
{
    let total = results.len();
    outcomes_in_scope(results, total)
}

/// Renders outcomes against an explicit repository scope. This is used by
/// commands whose result count is not the same as the number of child repos
/// involved, such as a move that produces one outcome per moved entry.
pub(crate) fn outcomes_in_scope(
    results: Vec<GitCommandResult>,
    total: usize
) -> Result<Rendered>
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

    let mut stdout = String::new();
    for message in grouped_messages(successes, total, SuffixPolicy::Truncated)
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

    groups
        .into_iter()
        .map(|(message, repos)| {
            let repos = repos.iter().map(String::as_str).collect::<Vec<_>>();
            format!("{message}{}", repo_list_suffix(&repos, total, policy))
        })
        .collect()
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
        let result = outcomes(vec![Ok(RepoOutcome::Success(None))]);

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
        let rendered = outcomes(results).unwrap();

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
        let rendered = outcomes(results).unwrap();

        // Assert
        assert_eq!(
            rendered.stdout,
            "Already up to date. \x1b[90m(backend)\x1b[0m\n"
        );
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
        let err = outcomes(results).unwrap_err();

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
        let err = outcomes(vec![error]).unwrap_err();

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
        let err = outcomes(results).unwrap_err();

        // Assert
        let failed = err.downcast::<crate::render::Failed>().unwrap();
        assert_eq!(failed.rendered.stdout, "pushed \x1b[90m(backend)\x1b[0m\n");
        assert_eq!(failed.message, "remote rejected \x1b[90m(frontend)\x1b[0m");
    }

    #[test]
    fn explicit_scope_deduplicates_repos_from_repeated_entry_outcomes()
    {
        // Arrange: two move entries failed in the same destination repo,
        // within a command whose scope also includes the source repo.
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
        let err = outcomes_in_scope(results, 2).unwrap_err();

        // Assert
        assert_eq!(
            err.to_string(),
            "unable to stage \x1b[90m(frontend)\x1b[0m"
        );
    }
}
