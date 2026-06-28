mod builder;

use crate::cli::CliContext;
use aptabase_rs::Builder;
use chrono::Utc;
use serde_json::json;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

const EVENT_NAME: &str = "command_finished";
const FLUSH_TIMEOUT: Duration = Duration::from_millis(500);
const APP_KEY: Option<&str> = option_env!("GITVMR_APTABASE_KEY");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEvent
{
    pub name: &'static str,
    pub success: bool,
    pub duration_ms: u128,
    pub flags: Vec<&'static str>,
    pub global_flags: Vec<&'static str>
}

/// Record a command event without affecting command behavior.
pub fn record(context: &mut CliContext, event: CommandEvent)
{
    let session_id = context.global_state.analytics.eval_session_id(Utc::now());

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

    let client = builder::build_analytics_client(
        Builder::new(app_key, env!("CARGO_PKG_VERSION")),
        session_id
    );
    let _ = client.track_event(EVENT_NAME, Some(props));
    let _ = runtime.block_on(async {
        tokio::time::timeout(FLUSH_TIMEOUT, client.flush()).await
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn event_keeps_expected_fields()
    {
        let event = CommandEvent {
            name: "rm",
            success: false,
            duration_ms: 42,
            flags: vec!["force"],
            global_flags: vec!["working_dir"]
        };

        assert_eq!(event.name, "rm");
        assert!(!event.success);
        assert_eq!(event.duration_ms, 42);
        assert_eq!(event.flags, ["force"]);
        assert_eq!(event.global_flags, ["working_dir"]);
    }
}
