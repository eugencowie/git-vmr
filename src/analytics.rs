use anyhow::Result;
use aptabase_rs::Builder;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::ffi::OsStr;
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
    if let Some(path) = env::var_os("GITVMR_ANALYTICS_LOG")
    {
        record_with_sink(config, state, event, Utc::now(), |_, props| {
            log_to_file(&path, &props)
        });
    }
    else if let Some(app_key) = APP_KEY
    {
        record_with_sink(config, state, event, Utc::now(), |session, props| {
            emit_to_aptabase(app_key, session, props)
        });
    }
    else
    {
        record_with_sink(config, state, event, Utc::now(), |_, _| Ok(()));
    }
}

/// Evaluate the session and hand the built event to the analytics sink
fn record_with_sink(
    config: &Analytics,
    state: &mut AnalyticsState,
    event: CommandEvent,
    now: DateTime<Utc>,
    sink: impl FnOnce(String, serde_json::Value) -> Result<()>
)
{
    // Disabled analytics record nothing
    if !config.enabled()
    {
        return;
    }

    let session_id = state.eval_session_id(now);

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

    // Analytics failures are always quiet
    let _ = sink(session_id, props);
}

/// Append event props to the analytics log file
fn log_to_file(path: &OsStr, props: &serde_json::Value) -> Result<()>
{
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{props}")?;
    Ok(())
}

/// Emit an event to the hosted collector
fn emit_to_aptabase(
    app_key: &str,
    session_id: String,
    props: serde_json::Value
) -> Result<()>
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;
    let client = Builder::new(app_key, env!("CARGO_PKG_VERSION"))
        .with_session_id(session_id)
        .build();
    client.track_event(EVENT_NAME, Some(props)).map_err(anyhow::Error::msg)?;
    runtime.block_on(async {
        tokio::time::timeout(FLUSH_TIMEOUT, client.flush()).await
    })?;
    Ok(())
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

    mod record_with_sink
    {
        use super::*;
        use std::cell::{Cell, RefCell};

        fn enabled() -> Analytics
        {
            Analytics { enabled: Some(true) }
        }

        fn event(name: &str) -> CommandEvent
        {
            CommandEvent {
                name: name.to_owned(),
                success: true,
                duration_ms: 1,
                flags: Vec::new(),
                global_flags: Vec::new()
            }
        }

        /// Record one event and return what reached the sink
        fn emit(
            state: &mut AnalyticsState,
            event: CommandEvent,
            now: DateTime<Utc>
        ) -> (String, serde_json::Value)
        {
            let emitted = RefCell::new(None);

            record_with_sink(
                &enabled(),
                state,
                event,
                now,
                |session, props| {
                    *emitted.borrow_mut() = Some((session, props));
                    Ok(())
                }
            );

            emitted.into_inner().expect("sink was not called")
        }

        #[test]
        fn disabled_record_does_not_update_state_or_emit()
        {
            let analytics = Analytics { enabled: Some(false) };
            let mut state = AnalyticsState {
                session_id: Some("session-1".to_owned()),
                last_activity: Some(now() - Duration::hours(4))
            };
            let original_state = state.clone();
            let emitted = Cell::new(false);

            record_with_sink(
                &analytics,
                &mut state,
                event("list"),
                now(),
                |_, _| {
                    emitted.set(true);
                    Ok(())
                }
            );

            assert!(!emitted.get());
            assert_eq!(state, original_state);
        }

        #[test]
        fn props_carry_name_success_and_duration()
        {
            let mut state = AnalyticsState::default();

            let (_, props) = emit(&mut state, event("status.list"), now());

            assert_eq!(props["name"], "status.list");
            assert_eq!(props["success"], true);
            assert_eq!(props["duration_ms"], 1);
        }

        #[test]
        fn explicit_flags_are_suffixed()
        {
            let mut state = AnalyticsState::default();
            let event = CommandEvent {
                flags: vec!["all".to_owned()],
                global_flags: vec!["directory".to_owned()],
                ..event("add")
            };

            let (_, props) = emit(&mut state, event, now());

            assert_eq!(props["all_flag"], true);
            assert_eq!(props["directory_global_flag"], true);
        }

        #[test]
        fn missing_session_selects_new_id()
        {
            let mut state = AnalyticsState::default();

            let (session_id, _) = emit(&mut state, event("list"), now());

            assert!(!session_id.is_empty());
            assert_eq!(state.session_id, Some(session_id));
            assert_eq!(state.last_activity, Some(now()));
        }

        #[test]
        fn active_session_reuses_id()
        {
            let mut state = AnalyticsState::default();

            let (first, _) = emit(&mut state, event("list"), now());
            let (second, _) =
                emit(&mut state, event("list"), now() + Duration::hours(3));

            assert_eq!(second, first);
            assert_eq!(state.last_activity, Some(now() + Duration::hours(3)));
        }

        #[test]
        fn expired_session_rotates_id()
        {
            let mut state = AnalyticsState::default();

            let (first, _) = emit(&mut state, event("list"), now());
            let (second, _) =
                emit(&mut state, event("list"), now() + Duration::hours(4));

            assert_ne!(second, first);
            assert_eq!(state.session_id, Some(second));
        }

        #[test]
        fn sink_failure_is_quiet_and_keeps_state()
        {
            let mut state = AnalyticsState::default();

            record_with_sink(
                &enabled(),
                &mut state,
                event("list"),
                now(),
                |_, _| anyhow::bail!("collector unreachable")
            );

            assert!(state.session_id.is_some());
            assert_eq!(state.last_activity, Some(now()));
        }
    }

    mod analytics_state
    {
        use super::*;

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
    }
}
