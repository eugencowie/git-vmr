use crate::git::{GitCommandResult, RepoMessage, RepoOutcome};
use crate::render::{Rendered, fail};
use anyhow::Result;

/// Renders aggregated per-repo outcomes: identical messages are grouped,
/// successes become stdout, and failures become the command's error.
pub fn outcomes(results: Vec<GitCommandResult>) -> Result<Rendered>
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
    for message in
        grouped_messages(successes, RepositoryFormat::NamesUntilLimit)
    {
        stdout.push_str(&message);
        stdout.push('\n');
    }

    let rendered = Rendered { stdout, stderr: String::new() };

    let mut rendered_errors =
        grouped_messages(failures, RepositoryFormat::NamesWithCount);
    rendered_errors.extend(errors.into_iter().map(|error| error.to_string()));

    if !rendered_errors.is_empty()
    {
        return Err(fail(rendered, rendered_errors.join("\n")));
    }

    Ok(rendered)
}

const REPOSITORY_NAME_LIMIT: usize = 5;

enum RepositoryFormat
{
    NamesUntilLimit,
    NamesWithCount
}

fn grouped_messages(
    messages: Vec<RepoMessage>,
    format: RepositoryFormat
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

fn repository_suffix(repos: &[String], format: &RepositoryFormat) -> String
{
    if repos.len() == 1
    {
        return format!("({})", repos[0]);
    }

    match format
    {
        RepositoryFormat::NamesUntilLimit
            if repos.len() > REPOSITORY_NAME_LIMIT =>
        {
            format!("({} repos)", repos.len())
        }
        RepositoryFormat::NamesUntilLimit => format!("({})", repos.join(", ")),
        RepositoryFormat::NamesWithCount
            if repos.len() > REPOSITORY_NAME_LIMIT =>
        {
            format!(
                "({} repos: {}, ...)",
                repos.len(),
                repos[..REPOSITORY_NAME_LIMIT].join(", ")
            )
        }
        RepositoryFormat::NamesWithCount => format!("({})", repos.join(", "))
    }
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
        let rendered =
            grouped_messages(messages, RepositoryFormat::NamesUntilLimit);

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
        let rendered =
            grouped_messages(messages, RepositoryFormat::NamesUntilLimit);

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
            grouped_messages(messages, RepositoryFormat::NamesWithCount);

        // Assert
        assert_eq!(rendered, vec![
            "remote rejected (6 repos: repo-1, repo-2, repo-3, repo-4, repo-5, ...)"
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
        assert_eq!(
            rendered.stdout,
            "Already up to date. (backend, frontend)\n"
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
            "error: branch not found (backend, frontend)\nfatal: transport failed"
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
        assert_eq!(failed.rendered.stdout, "pushed (backend)\n");
        assert_eq!(failed.message, "remote rejected (frontend)");
    }
}
