use aptabase_rs::Builder;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration as StdDuration;

const EVENT_NAME: &str = "command_finished";
const FLUSH_TIMEOUT: StdDuration = StdDuration::from_millis(500);
const APP_KEY: Option<&str> = option_env!("GITVMR_APTABASE_KEY");
const SESSION_WINDOW: Duration = Duration::hours(4);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Analytics
{
    /// Whether usage analytics are enabled
    enabled: Option<bool>
}

impl Analytics
{
    /// Whether analytics are enabled
    fn enabled(&self) -> bool
    {
        self.enabled_for_major_version(env!("CARGO_PKG_VERSION_MAJOR"))
    }

    /// Whether analytics are enabled for a major version
    fn enabled_for_major_version(&self, major_version: &str) -> bool
    {
        self.enabled.unwrap_or(major_version == "0")
    }
}

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
    fn eval_session_id(&mut self, now: DateTime<Utc>) -> String
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEvent
{
    pub name: String,
    pub success: bool,
    pub duration_ms: u128,
    pub flags: Vec<String>,
    pub global_flags: Vec<String>
}

/// Record a command event without affecting command behavior.
pub fn record(
    config: &Analytics,
    state: &mut AnalyticsState,
    event: CommandEvent
)
{
    // Disabled analytics record nothing
    if !config.enabled()
    {
        return;
    }

    let session_id = state.eval_session_id(Utc::now());

    let mut props = json!({
        "name": event.name,
        "success": event.success,
        "duration_ms": event.duration_ms
    });

    if let Some(props) = props.as_object_mut()
    {
        for flag in event.flags
        {
            props.insert(format!("{flag}_flag"), true.into());
        }

        for flag in event.global_flags
        {
            props.insert(format!("{flag}_global_flag"), true.into());
        }
    }

    if let Some(path) = env::var_os("GITVMR_ANALYTICS_LOG")
    {
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| writeln!(file, "{props}"));
        return;
    }

    let Some(app_key) = APP_KEY
    else
    {
        return;
    };

    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
    else
    {
        return;
    };

    let client = Builder::new(app_key, env!("CARGO_PKG_VERSION"))
        .with_session_id(session_id)
        .build();
    let _ = client.track_event(EVENT_NAME, Some(props));
    let _ = runtime.block_on(async {
        tokio::time::timeout(FLUSH_TIMEOUT, client.flush()).await
    });
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

    mod analytics
    {
        use super::*;

        #[test]
        fn missing_enabled_follows_version_default()
        {
            let analytics = Analytics::default();

            assert!(analytics.enabled_for_major_version("0"));
            assert!(!analytics.enabled_for_major_version("1"));
        }

        #[test]
        fn explicit_enabled_overrides_version_default()
        {
            assert!(
                !Analytics { enabled: Some(false) }
                    .enabled_for_major_version("0")
            );
            assert!(
                Analytics { enabled: Some(true) }
                    .enabled_for_major_version("1")
            );
        }
    }

    mod analytics_state
    {
        use super::*;

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
}
