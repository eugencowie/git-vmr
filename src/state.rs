mod updates;

use crate::cli::APP_NAME;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs};
pub use updates::UpdateState;

const APP_NAME: &str = "git-vmr";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalState
{
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
        Self::load_from_path(&state_path)
    }

    /// Load global state from path
    pub fn load_from_path(path: &Path) -> Result<Self>
    {
        match fs::read_to_string(path)
        {
            Ok(contents) => toml::from_str(&contents)
                .with_context(|| format!("failed to parse {}", path.display())),

            // Missing global state uses defaults
            Err(err) if err.kind() == ErrorKind::NotFound =>
                Ok(Self::default()),

            Err(err) => Err(err)
                .with_context(|| format!("failed to read {}", path.display()))
        }
    }

    pub fn save(&self, path: &Path) -> Result<()>
    {
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
            .with_context(|| format!("failed to write {}", path.display()))
    }
}

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
            }
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
        let state = GlobalState {
            updates: UpdateState {
                last_check: Some(now()),
                last_available: Some("1.2.3".to_owned())
            }
        };

        // Act
        state.save(&path).unwrap();
        let loaded = GlobalState::load_from_path(&path).unwrap();

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
            GlobalState::load_from_path(&tmp.path().join("missing.toml"))
                .unwrap();

        // Assert
        assert_eq!(state, GlobalState::default());
    }

    #[test]
    fn load_rejects_malformed_file()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("state.toml");
        fs::write(&path, "[updates]\nlast_check =").unwrap();

        // Act
        let err = GlobalState::load_from_path(&path).unwrap_err();

        // Assert
        assert!(err.to_string().contains("failed to parse"));
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
