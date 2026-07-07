use crate::config::GlobalConfig;
use crate::git::Git;
use crate::state::GlobalState;
use crate::store::FileStore;
use anyhow::{Context, Result, bail};
use std::env;
use std::path::PathBuf;

pub struct CliContext
{
    /// Display name of the application
    pub display_name: String,

    /// Working directory for the command
    pub working_dir: PathBuf,

    /// Global configuration
    pub global_config: FileStore<GlobalConfig>,

    /// Global runtime state
    pub global_state: FileStore<GlobalState>,

    /// Warnings raised while loading context data
    pub warnings: Vec<String>,

    /// Git operations adapter
    pub git: Git
}

impl CliContext
{
    /// Build context for command execution
    pub fn new(
        display_name: &str,
        working_dir: &Option<PathBuf>
    ) -> Result<Self>
    {
        let global_config = FileStore::load(GlobalConfig::resolve_path(None)?)?;
        let (global_state, state_warning) =
            FileStore::load_or_default(GlobalState::resolve_path(None)?);

        // Resolve inputs needed by commands
        Ok(Self {
            display_name: display_name.to_owned(),
            working_dir: Self::resolve_working_dir(working_dir)?,
            global_config,
            global_state,
            warnings: state_warning.into_iter().collect(),
            git: Git::subprocess()
        })
    }

    /// Save changes to the context
    pub fn save(&mut self) -> Result<()>
    {
        self.global_state.save()
    }

    /// Resolve working directory for command execution
    fn resolve_working_dir(working_dir: &Option<PathBuf>) -> Result<PathBuf>
    {
        // Parse working directory from argument
        if let Some(working_dir) = working_dir
        {
            // Get absolute path
            let absolute_path =
                working_dir.canonicalize().with_context(|| {
                    format!(
                        "fatal: cannot change to '{}'",
                        working_dir.display()
                    )
                })?;

            // Ensure the path is a directory
            if !absolute_path.is_dir()
            {
                bail!(
                    "fatal: cannot change to '{}': Not a directory",
                    working_dir.display()
                );
            }

            return Ok(absolute_path);
        }

        // If none provided, use current working directory
        env::current_dir().context("fatal: failed to get current directory")
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs;

    #[test]
    fn resolves_working_dir_argument_to_canonical_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let working_dir = Some(nested.join("..").join("nested"));

        // Act
        let context = CliContext::new("git vmr", &working_dir).unwrap();

        // Assert
        assert_eq!(context.working_dir, nested.canonicalize().unwrap());
    }

    #[test]
    fn rejects_working_dir_argument_that_is_not_a_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("file");
        fs::write(&file, "").unwrap();
        let working_dir = Some(file.clone());

        // Act
        let Err(err) = CliContext::new("git vmr", &working_dir)
        else
        {
            panic!("expected working directory validation to fail");
        };

        // Assert
        assert_eq!(
            err.to_string(),
            format!(
                "fatal: cannot change to '{}': Not a directory",
                file.display()
            )
        );
    }

    #[test]
    fn reports_missing_working_dir_argument_path()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("missing");
        let working_dir = Some(missing.clone());

        // Act
        let Err(err) = CliContext::new("git vmr", &working_dir)
        else
        {
            panic!("expected working directory validation to fail");
        };

        // Assert
        assert_eq!(
            err.to_string(),
            format!("fatal: cannot change to '{}'", missing.display())
        );
    }

    #[test]
    fn uses_current_dir_when_working_dir_argument_is_missing()
    {
        // Arrange
        let working_dir = None;

        // Act
        let context = CliContext::new("git vmr", &working_dir).unwrap();

        // Assert
        assert_eq!(context.working_dir, env::current_dir().unwrap());
    }
}
