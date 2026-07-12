use crate::config::Frequency;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateState
{
    /// Last time an update check was attempted
    pub last_check: Option<DateTime<Utc>>,

    /// Last available version reported by update checks
    pub last_available: Option<String>
}

impl UpdateState
{
    /// Whether an update check is due
    pub fn is_due(&self, frequency: Frequency, now: DateTime<Utc>) -> bool
    {
        // Disabled update checks are never due
        let Some(interval) = frequency.interval()
        else
        {
            return false;
        };

        // Missing or expired check state is due
        self.last_check.is_none_or(|last| {
            now.signed_duration_since(last)
                .to_std()
                .is_ok_and(|elapsed| elapsed >= interval)
        })
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use chrono::Duration;
    use std::time::Duration as StdDuration;

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
        let state = UpdateState::default();

        // Assert
        assert_eq!(state.last_check, None);
        assert_eq!(state.last_available, None);
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        // Arrange
        let state = UpdateState {
            last_check: Some(now()),
            last_available: Some("1.2.3".to_owned())
        };

        // Act
        let toml = toml::to_string(&state).unwrap();

        // Assert
        assert_eq!(
            toml,
            "last_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\"\n"
        );
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        // Act
        let state: UpdateState = toml::from_str(
            "last_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\""
        )
        .unwrap();

        // Assert
        assert_eq!(state.last_check, Some(now()));
        assert_eq!(state.last_available, Some("1.2.3".to_owned()));
    }

    #[test]
    fn missing_state_is_due()
    {
        // Arrange
        let state = UpdateState::default();

        // Act
        let due = state.is_due(
            Frequency::Every(StdDuration::from_secs(60 * 60 * 24)),
            now()
        );

        // Assert
        assert!(due);
    }

    #[test]
    fn recent_state_is_not_due()
    {
        // Arrange
        let state = UpdateState {
            last_check: Some(now() - Duration::hours(23)),
            last_available: None
        };

        // Act
        let due = state.is_due(
            Frequency::Every(StdDuration::from_secs(60 * 60 * 24)),
            now()
        );

        // Assert
        assert!(!due);
    }

    #[test]
    fn expired_state_is_due()
    {
        // Arrange
        let state = UpdateState {
            last_check: Some(now() - Duration::days(31)),
            last_available: None
        };

        // Act
        let due = state.is_due(
            Frequency::Every(StdDuration::from_secs(60 * 60 * 24 * 30)),
            now()
        );

        // Assert
        assert!(due);
    }

    #[test]
    fn never_frequency_is_not_due()
    {
        // Arrange
        let state = UpdateState::default();

        // Act
        let due = state.is_due(Frequency::Never, now());

        // Assert
        assert!(!due);
    }

    #[test]
    fn future_attempt_is_not_due()
    {
        // Arrange
        let state = UpdateState {
            last_check: Some(now() + Duration::hours(1)),
            last_available: None
        };

        // Act
        let due = state
            .is_due(Frequency::Every(StdDuration::from_secs(60 * 60)), now());

        // Assert
        assert!(!due);
    }
}
