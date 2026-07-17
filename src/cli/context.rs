use crate::config::GlobalConfig;
use crate::git::Git;
use crate::state::GlobalState;
use crate::store::FileStore;
use anyhow::{Context, Result, bail};
use std::env;
use std::io::{self, IsTerminal};
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

    /// Whether stdout is a terminal — the one TTY fact gating colour and
    /// paging decisions, so they can never disagree
    pub stdout_is_tty: bool,

    /// Warnings raised while loading context data. Private so the
    /// constructor is the only way to build a context outside this module.
    warnings: Vec<String>,

    /// Git operations adapter
    pub git: Git
}

impl CliContext
{
    /// Build context for command execution from resolved file paths
    pub fn new(
        display_name: &str,
        working_dir: &Option<PathBuf>,
        config_path: PathBuf,
        state_path: PathBuf
    ) -> Result<Self>
    {
        let global_config = FileStore::load(config_path)?;
        let (global_state, state_warning) =
            FileStore::load_or_default(state_path);

        // Resolve inputs needed by commands
        Ok(Self {
            display_name: display_name.to_owned(),
            working_dir: Self::resolve_working_dir(working_dir)?,
            global_config,
            global_state,
            stdout_is_tty: io::stdout().is_terminal(),
            warnings: state_warning.into_iter().collect(),
            git: Git::subprocess()
        })
    }

    /// A context rooted at `working_dir`, with stores that touch nothing
    /// until saved. Tests that exercise git inject a scripted fake.
    #[cfg(test)]
    pub(crate) fn for_tests(working_dir: &std::path::Path, git: Git) -> Self
    {
        let (global_state, _) =
            FileStore::load_or_default(working_dir.join("state.toml"));

        Self {
            display_name: "git vmr".to_owned(),
            working_dir: working_dir.to_path_buf(),
            global_config: FileStore::load(working_dir.join("config.toml"))
                .expect("missing global config loads as defaults"),
            global_state,
            stdout_is_tty: false,
            warnings: Vec::new(),
            git
        }
    }

    /// Warnings raised while loading context data
    pub fn warnings(&self) -> &[String]
    {
        &self.warnings
    }

    /// Save changes to the context
    pub fn save(&mut self) -> Result<()>
    {
        self.global_config.save()?;
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
    use std::path::Path;

    fn new_context(
        tmp: &Path,
        working_dir: &Option<PathBuf>
    ) -> Result<CliContext>
    {
        CliContext::new(
            "git vmr",
            working_dir,
            tmp.join("config.toml"),
            tmp.join("state.toml")
        )
    }

    #[test]
    fn saves_global_config_and_state_changes()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        let state_path = tmp.path().join("state.toml");
        let mut context = CliContext {
            display_name: "git vmr".to_owned(),
            working_dir: PathBuf::new(),
            global_config: FileStore::new(
                config_path.clone(),
                GlobalConfig::default()
            ),
            global_state: FileStore::new(
                state_path.clone(),
                GlobalState::default()
            ),
            stdout_is_tty: false,
            warnings: vec![],
            git: Git::subprocess()
        };
        context.global_config.core.version = 1;
        context.global_state.updates.last_available = Some("1.2.3".to_owned());

        // Act
        context.save().unwrap();

        // Assert
        let config = FileStore::<GlobalConfig>::load(config_path).unwrap();
        let state = FileStore::<GlobalState>::load(state_path).unwrap();
        assert_eq!(config.core.version, 1);
        assert_eq!(state.updates.last_available, Some("1.2.3".to_owned()));
    }

    #[test]
    fn resolves_working_dir_argument_to_canonical_directory()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let working_dir = Some(nested.join("..").join("nested"));

        // Act
        let context = new_context(tmp.path(), &working_dir).unwrap();

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
        let Err(err) = new_context(tmp.path(), &working_dir)
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
        let Err(err) = new_context(tmp.path(), &working_dir)
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
        let tmp = tempfile::tempdir().unwrap();
        let context = new_context(tmp.path(), &working_dir).unwrap();

        // Assert
        assert_eq!(context.working_dir, env::current_dir().unwrap());
    }

    #[test]
    fn fails_on_malformed_global_config()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("config.toml"), "not = valid = toml")
            .unwrap();

        // Act
        let Err(err) = new_context(tmp.path(), &None)
        else
        {
            panic!("expected strict global config load to fail");
        };

        // Assert
        assert_eq!(
            err.to_string(),
            format!(
                "failed to parse {}",
                tmp.path().join("config.toml").display()
            )
        );
    }

    #[test]
    fn defaults_and_warns_on_malformed_global_state()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("state.toml"), "not = valid = toml").unwrap();

        // Act
        let context = new_context(tmp.path(), &None).unwrap();

        // Assert
        assert_eq!(*context.global_state, GlobalState::default());
        assert_eq!(context.warnings().len(), 1);
        assert!(context.warnings()[0].contains("using defaults"));
    }
}
