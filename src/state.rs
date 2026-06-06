use crate::config::Frequency;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCheckState
{
    pub last_attempted_check: Option<DateTime<Utc>>,
    pub last_available_version: Option<String>
}

impl UpdateCheckState
{
    pub fn load(path: &Path) -> Result<Self>
    {
        match fs::read_to_string(path)
        {
            Ok(contents) => toml::from_str(&contents)
                .with_context(|| format!("failed to parse {}", path.display())),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound =>
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
        let contents = toml::to_string(self)?;
        fs::write(path, contents)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn is_due(&self, frequency: Frequency, now: DateTime<Utc>) -> bool
    {
        let Some(interval) = frequency.interval()
        else
        {
            return false;
        };
        self.last_attempted_check
            .is_none_or(|last| now.signed_duration_since(last) >= interval)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use chrono::Duration;

    fn now() -> DateTime<Utc>
    {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn roundtrips()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("state").join("update.toml");
        let state = UpdateCheckState {
            last_attempted_check: Some(now()),
            last_available_version: Some("1.2.3".to_owned())
        };

        // Act
        state.save(&path).unwrap();
        let loaded = UpdateCheckState::load(&path).unwrap();

        // Assert
        assert_eq!(loaded, state);
    }

    #[test]
    fn missing_state_is_due()
    {
        // Arrange
        let state = UpdateCheckState::default();

        // Act
        let due = state.is_due(Frequency::Daily, now());

        // Assert
        assert!(due);
    }

    #[test]
    fn recent_state_is_not_due()
    {
        // Arrange
        let state = UpdateCheckState {
            last_attempted_check: Some(now() - Duration::hours(23)),
            last_available_version: None
        };

        // Act
        let due = state.is_due(Frequency::Daily, now());

        // Assert
        assert!(!due);
    }

    #[test]
    fn expired_state_is_due()
    {
        // Arrange
        let state = UpdateCheckState {
            last_attempted_check: Some(now() - Duration::days(31)),
            last_available_version: None
        };

        // Act
        let due = state.is_due(Frequency::Monthly, now());

        // Assert
        assert!(due);
    }
}
