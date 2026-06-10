mod updates;

use crate::cli::APP_NAME;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs};
pub use updates::UpdateState;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalState
{
    /// Path to the state file
    #[serde(skip)]
    path: Option<PathBuf>,

    /// Whether the state has unsaved changes
    #[serde(skip)]
    dirty: bool,

    /// Software update state
    pub updates: UpdateState
}

impl GlobalState
{
    /// Load global state
    pub fn load() -> Result<Self>
    {
        // Resolve state path
        let state_path = Self::resolve_path(None)?;

        // Load state from path
        Ok(Self::load_from_path(&state_path))
    }

    /// Load global state from path
    pub fn load_from_path(path: &Path) -> Self
    {
        // Read and parse state file
        match fs::read_to_string(path)
        {
            Ok(contents) =>
            {
                let mut state: Self = match toml::from_str(&contents)
                {
                    Ok(state) => state,
                    Err(err) =>
                    {
                        eprintln!(
                            "warning: failed to parse {}: {err:#}; using default state",
                            path.display()
                        );
                        Self { dirty: true, ..Self::default() }
                    }
                };
                state.path = Some(path.to_owned());
                state
            }

            // Missing global state uses defaults
            Err(err) if err.kind() == ErrorKind::NotFound =>
                Self { path: Some(path.to_path_buf()), ..Self::default() },

            // Any other error prints warning and uses defaults
            Err(err) =>
            {
                eprintln!(
                    "warning: failed to read {}: {err:#}; using default state",
                    path.display()
                );
                Self {
                    path: Some(path.to_path_buf()),
                    dirty: true,
                    ..Self::default()
                }
            }
        }
    }

    /// Mark the state as having unsaved changes
    pub fn mark_dirty(&mut self)
    {
        self.dirty = true;
    }

    /// Save global state
    pub fn save(&mut self) -> Result<()>
    {
        // Skip save if state is not dirty
        if !self.dirty
        {
            return Ok(());
        }

        // Get resolved state path
        let path = self.path.as_deref().context(
            "failed to write global state before resolving state path"
        )?;

        // Create parent directory if needed
        if let Some(parent) = path.parent()
        {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create {}", parent.display())
            })?;
        }

        // Serialize and write state file
        let contents = toml::to_string(self).with_context(|| {
            format!("failed to serialize {}", path.display())
        })?;
        fs::write(path, contents)
            .with_context(|| format!("failed to write {}", path.display()))?;
        self.dirty = false;
        Ok(())
    }

    /// Resolve global state file path
    fn resolve_path(state_dir: Option<PathBuf>) -> Result<PathBuf>
    {
        // Resolve state root directory
        let state_root = state_dir
            .or_else(|| env::var_os("GIT_VMR_STATE_DIR").map(PathBuf::from))
            .or_else(dirs::state_dir)
            .or_else(dirs::data_local_dir)
            .context("failed to resolve user state directory")?;

        // Build app-specific state path
        Ok(state_root.join(APP_NAME).join("state.toml"))
    }
}

impl PartialEq for GlobalState
{
    fn eq(&self, other: &Self) -> bool
    {
        // Ignore file path when comparing state
        self.updates == other.updates
    }
}

impl Eq for GlobalState {}

#[cfg(test)]
mod tests
{
    use super::*;
    use chrono::{DateTime, Utc};

    fn now() -> DateTime<Utc>
    {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn default_has_default_values()
    {
        // Act
        let state = GlobalState::default();

        // Assert
        assert_eq!(state.updates, UpdateState::default());
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        // Arrange
        let state = GlobalState {
            updates: UpdateState {
                last_check: Some(now()),
                last_available: Some("1.2.3".to_owned())
            },
            ..GlobalState::default()
        };

        // Act
        let toml = toml::to_string(&state).unwrap();

        // Assert
        assert_eq!(
            toml,
            "[updates]\nlast_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\"\n"
        );
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        // Act
        let state: GlobalState = toml::from_str(
            "[updates]\nlast_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\""
        )
        .unwrap();

        // Assert
        assert_eq!(state.updates.last_check, Some(now()));
        assert_eq!(state.updates.last_available, Some("1.2.3".to_owned()));
    }

    #[test]
    fn from_str_defaults_missing_updates_section()
    {
        // Act
        let state: GlobalState = toml::from_str("").unwrap();

        // Assert
        assert_eq!(state.updates, UpdateState::default());
    }

    #[test]
    fn roundtrips()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("state").join("state.toml");
        let mut state = GlobalState::load_from_path(&path);
        state.updates.last_check = Some(now());
        state.updates.last_available = Some("1.2.3".to_owned());
        state.mark_dirty();

        // Act
        state.save().unwrap();
        let loaded = GlobalState::load_from_path(&path);

        // Assert
        assert_eq!(loaded, state);
    }

    #[test]
    fn load_missing_file_returns_default()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let state =
            GlobalState::load_from_path(&tmp.path().join("missing.toml"));

        // Assert
        assert_eq!(state, GlobalState::default());
    }

    #[test]
    fn load_malformed_file_returns_default()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("state.toml");
        fs::write(&path, "[updates]\nlast_check =").unwrap();

        // Act
        let state = GlobalState::load_from_path(&path);

        // Assert
        assert_eq!(state, GlobalState::default());

        // Recovery keeps the path so the invalid file can be replaced
        let mut state = state;
        state.updates.last_available = Some("1.2.3".to_owned());
        state.save().unwrap();
        assert_eq!(
            GlobalState::load_from_path(&path).updates.last_available,
            Some("1.2.3".to_owned())
        );
    }

    #[test]
    fn path_uses_user_level_state_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();

        // Act
        let path =
            GlobalState::resolve_path(Some(tmp.path().join("state"))).unwrap();

        // Assert
        assert_eq!(
            path,
            tmp.path().join("state").join("git-vmr").join("state.toml")
        );
    }
}
