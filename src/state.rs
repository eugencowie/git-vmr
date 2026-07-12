use crate::analytics::AnalyticsState;
use crate::cli::APP_NAME;
use crate::updates::UpdateState;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalState
{
    /// Usage analytics state
    pub analytics: AnalyticsState,

    /// Software update state
    pub updates: UpdateState
}

impl GlobalState
{
    /// Resolve global state file path
    pub fn resolve_path(state_dir: Option<PathBuf>) -> Result<PathBuf>
    {
        // Resolve state root directory
        let state_root = state_dir
            .or_else(|| env::var_os("GITVMR_STATE_DIR").map(PathBuf::from))
            .or_else(dirs::state_dir)
            .or_else(dirs::data_local_dir)
            .context("failed to resolve user state directory")?;

        // Build app-specific state path
        Ok(state_root.join(APP_NAME).join("state.toml"))
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
        assert_eq!(state.analytics, AnalyticsState::default());
        assert_eq!(state.updates, UpdateState::default());
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        // Arrange
        let mut state = GlobalState::default();
        state.analytics.session_id = Some("session-1".to_owned());
        state.analytics.last_activity = Some(now());
        state.updates.last_check = Some(now());
        state.updates.last_available = Some("1.2.3".to_owned());

        // Act
        let toml = toml::to_string(&state).unwrap();

        // Assert
        assert_eq!(
            toml,
            "[analytics]\nsession_id = \"session-1\"\nlast_activity = \"2026-06-06T12:00:00Z\"\n\n[updates]\nlast_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\"\n"
        );
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        // Act
        let state: GlobalState = toml::from_str(
            "[analytics]\nsession_id = \"session-1\"\nlast_activity = \"2026-06-06T12:00:00Z\"\n\n[updates]\nlast_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\""
        )
        .unwrap();

        // Assert
        assert_eq!(state.analytics.session_id, Some("session-1".to_owned()));
        assert_eq!(state.analytics.last_activity, Some(now()));
        assert_eq!(state.updates.last_check, Some(now()));
        assert_eq!(state.updates.last_available, Some("1.2.3".to_owned()));
    }

    #[test]
    fn from_str_defaults_missing_updates_section()
    {
        // Act
        let state: GlobalState = toml::from_str("").unwrap();

        // Assert
        assert_eq!(state.analytics, AnalyticsState::default());
        assert_eq!(state.updates, UpdateState::default());
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
