use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

const SESSION_WINDOW: Duration = Duration::hours(4);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyticsState
{
    /// Active analytics session ID
    pub session_id: Option<String>,

    /// Last analytics activity time
    pub last_activity: Option<DateTime<Utc>>
}

impl AnalyticsState
{
    /// Select the active session ID and refresh activity
    pub fn eval_session_id(&mut self, now: DateTime<Utc>) -> String
    {
        let session_id = match (&self.session_id, self.last_activity)
        {
            (Some(session_id), Some(last_activity))
                if now.signed_duration_since(last_activity)
                    < SESSION_WINDOW =>
                session_id.clone(),

            _ => aptabase_rs::new_session_id()
        };

        self.session_id = Some(session_id.clone());
        self.last_activity = Some(now);
        session_id
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn now() -> DateTime<Utc>
    {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn default_has_default_values()
    {
        let state = AnalyticsState::default();

        assert_eq!(state.session_id, None);
        assert_eq!(state.last_activity, None);
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        let state = AnalyticsState {
            session_id: Some("session-1".to_owned()),
            last_activity: Some(now())
        };

        let toml = toml::to_string(&state).unwrap();

        assert_eq!(
            toml,
            "session_id = \"session-1\"\nlast_activity = \"2026-06-06T12:00:00Z\"\n"
        );
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        let state: AnalyticsState = toml::from_str(
            "session_id = \"session-1\"\nlast_activity = \"2026-06-06T12:00:00Z\""
        )
        .unwrap();

        assert_eq!(state.session_id, Some("session-1".to_owned()));
        assert_eq!(state.last_activity, Some(now()));
    }

    #[test]
    fn missing_session_selects_new_id()
    {
        let mut state = AnalyticsState::default();

        let session_id = state.eval_session_id(now());

        assert!(!session_id.is_empty());
        assert_eq!(state.session_id, Some(session_id));
        assert_eq!(state.last_activity, Some(now()));
    }

    #[test]
    fn active_session_reuses_id()
    {
        let mut state = AnalyticsState {
            session_id: Some("session-1".to_owned()),
            last_activity: Some(now() - Duration::hours(3))
        };

        let session_id = state.eval_session_id(now());

        assert_eq!(session_id, "session-1");
        assert_eq!(state.last_activity, Some(now()));
    }

    #[test]
    fn expired_session_rotates_id()
    {
        let mut state = AnalyticsState {
            session_id: Some("session-1".to_owned()),
            last_activity: Some(now() - Duration::hours(4))
        };

        let session_id = state.eval_session_id(now());

        assert_ne!(session_id, "session-1");
        assert_eq!(state.session_id, Some(session_id));
        assert_eq!(state.last_activity, Some(now()));
    }
}
