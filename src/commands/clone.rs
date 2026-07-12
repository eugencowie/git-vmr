use crate::cli::SilentError;
use crate::git::Git;
use anyhow::Result;
use std::path::Path;

pub fn clone(
    git: &Git,
    working_dir: &Path,
    repository: &str,
    directory: Option<&Path>
) -> Result<()>
{
    let status = git.clone(working_dir, repository, directory)?;

    if !status.success()
    {
        return Err(SilentError.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::git::ScriptedFake;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn clone_passes_repository_and_directory_to_git()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on_interactive(["clone", "https://example.com/repo", "dir"], 0);
        let git = Git::with(fake);

        // Act
        let result = clone(
            &git,
            Path::new("/vmr"),
            "https://example.com/repo",
            Some(Path::new("dir"))
        );

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn clone_records_invocation_in_working_dir()
    {
        // Arrange
        let fake = Arc::new(
            ScriptedFake::new()
                .on_interactive(["clone", "https://example.com/repo"], 0)
        );
        let git = Git::with(Arc::clone(&fake));

        // Act
        clone(&git, Path::new("/vmr"), "https://example.com/repo", None)
            .unwrap();

        // Assert
        let calls = fake.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].path, PathBuf::from("/vmr"));
    }

    #[test]
    fn clone_failure_is_silent_because_git_already_reported_it()
    {
        // Arrange
        let fake = ScriptedFake::new()
            .on_interactive(["clone", "https://example.com/repo"], 128);
        let git = Git::with(fake);

        // Act
        let error =
            clone(&git, Path::new("/vmr"), "https://example.com/repo", None)
                .unwrap_err();

        // Assert
        assert!(error.is::<SilentError>());
    }
}
