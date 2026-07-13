mod frequency;

use crate::cli::APP_NAME;
use crate::state::GlobalState;
use crate::store::FileStore;
use anyhow::Result;
use axoupdater::AxoUpdater;
use chrono::{DateTime, Utc};
pub use frequency::Frequency;
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

const QUERY_TIMEOUT: StdDuration = StdDuration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Updates
{
    /// How often to check for updates
    #[serde(rename = "checkfrequency")]
    pub check_frequency: Frequency
}

impl Default for Updates
{
    /// Default update configuration
    fn default() -> Self
    {
        // Check for updates daily by default
        Self { check_frequency: Frequency::from_days(1) }
    }
}

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
    fn is_due(&self, frequency: Frequency, now: DateTime<Utc>) -> bool
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

/// Check for updates and return a notice if a new version is available
pub fn check(
    check_frequency: Frequency,
    state: &mut FileStore<GlobalState>
) -> Option<String>
{
    check_with_query(check_frequency, state, Utc::now(), query_with_axoupdater)
}

/// Run a scheduled update check
fn check_with_query(
    check_frequency: Frequency,
    state: &mut FileStore<GlobalState>,
    now: DateTime<Utc>,
    mut query: impl FnMut() -> Result<Option<String>>
) -> Option<String>
{
    if !state.updates.is_due(check_frequency, now)
    {
        return None;
    }

    // Record the attempt before the query so failures don't retry-storm
    state.updates.last_check = Some(now);
    state.save().ok()?;

    let version = query().ok()??;
    state.updates.last_available = Some(version.clone());

    Some(format!("\nA new git-vmr version is available: {version}"))
}

/// Query for updates using axoupdater
fn query_with_axoupdater() -> Result<Option<String>>
{
    // Configure updater
    let mut updater = AxoUpdater::new_for(APP_NAME);
    updater.load_receipt()?;

    // Run asynchronous query on a local runtime
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;
    let latest = runtime.block_on(async {
        tokio::time::timeout(QUERY_TIMEOUT, async {
            let update_needed = updater.is_update_needed().await?;
            if update_needed
            {
                updater
                    .query_new_version()
                    .await
                    .map(|version| version.cloned())
            }
            else
            {
                Ok(None)
            }
        })
        .await
    })??;

    Ok(latest.map(|version| version.to_string()))
}

#[cfg(test)]
mod tests
{
    use super::*;
    use anyhow::bail;
    use chrono::Duration;
    use std::cell::Cell;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn run_with_store(
        frequency: Frequency,
        state_file: &Path,
        now: DateTime<Utc>,
        query: impl FnMut() -> Result<Option<String>>
    ) -> Option<String>
    {
        let (mut state, _) =
            FileStore::<GlobalState>::load_or_default(state_file.to_path_buf());
        let notice = check_with_query(frequency, &mut state, now, query);
        state.save().ok()?;
        notice
    }

    fn state_file(tmp: &tempfile::TempDir) -> PathBuf
    {
        tmp.path().join("state").join(APP_NAME).join("state.toml")
    }

    fn now() -> DateTime<Utc>
    {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn daily() -> Frequency
    {
        Frequency::from_days(1)
    }

    #[test]
    fn never_skips_update_checks()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let calls = Cell::new(0);

        // Act
        let notice =
            run_with_store(Frequency::Never, &state_file, now(), || {
                calls.set(calls.get() + 1);
                Ok(Some("9.0.0".into()))
            });

        // Assert
        assert_eq!(notice, None);
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn records_attempt_before_query()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || {
            let (state, _) =
                FileStore::<GlobalState>::load_or_default(state_file.clone());
            assert_eq!(state.updates.last_check, Some(now()));
            bail!("network failed")
        });

        // Assert
        assert_eq!(notice, None);
    }

    #[test]
    fn invalid_state_uses_defaults_and_runs_query()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        fs::create_dir_all(state_file.parent().unwrap()).unwrap();
        fs::write(&state_file, "[updates]\nlast_check =").unwrap();
        let calls = Cell::new(0);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || {
            calls.set(calls.get() + 1);
            Ok(Some("1.2.3".into()))
        });

        // Assert
        assert_eq!(
            notice,
            Some("\nA new git-vmr version is available: 1.2.3".to_owned())
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(
            FileStore::<GlobalState>::load_or_default(state_file)
                .0
                .updates
                .last_available,
            Some("1.2.3".to_owned())
        );
    }

    #[test]
    fn attempt_save_failure_is_quiet_and_skips_query()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        fs::create_dir_all(&state_file).unwrap();
        let calls = Cell::new(0);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || {
            calls.set(calls.get() + 1);
            Ok(Some("1.2.3".into()))
        });

        // Assert
        assert_eq!(notice, None);
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn available_version_save_failure_is_quiet()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let calls = Cell::new(0);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || {
            calls.set(calls.get() + 1);
            fs::remove_file(&state_file).unwrap();
            fs::create_dir(&state_file).unwrap();
            Ok(Some("1.2.3".into()))
        });

        // Assert
        assert_eq!(notice, None);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn new_version_prints_version_only_notice()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || {
            Ok(Some("1.2.3".into()))
        })
        .unwrap();

        // Assert
        assert_eq!(notice, "\nA new git-vmr version is available: 1.2.3");
    }

    #[test]
    fn current_version_is_quiet()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || Ok(None));

        // Assert
        assert_eq!(notice, None);
    }

    #[test]
    fn query_failure_is_quiet_and_non_fatal()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let calls = Cell::new(0);

        // Act
        let notice = run_with_store(daily(), &state_file, now(), || {
            calls.set(calls.get() + 1);
            bail!("timeout")
        });

        // Assert
        assert_eq!(notice, None);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn version_type_from_axoupdater_is_usable()
    {
        // Act
        let version: axoupdater::Version = "1.2.3".parse().unwrap();

        // Assert
        assert_eq!(version.to_string(), "1.2.3");
    }

    mod updates
    {
        use super::*;

        #[test]
        fn default_configuration_checks_daily()
        {
            // Act
            let updates = Updates::default();

            // Assert
            assert_eq!(updates.check_frequency, Frequency::from_days(1));
        }

        #[test]
        fn to_string_serializes_to_toml()
        {
            let updates = Updates::default();

            let toml = toml::to_string(&updates).unwrap();

            assert_eq!(toml, "checkfrequency = \"1day\"\n");
        }

        #[test]
        fn from_str_deserializes_from_toml()
        {
            let updates: Updates =
                toml::from_str("checkfrequency = \"1 week\"").unwrap();

            assert_eq!(updates.check_frequency, Frequency::from_days(7));
        }
    }

    mod update_state
    {
        use super::*;
        use std::time::Duration as StdDuration;

        #[test]
        fn to_string_serializes_to_toml()
        {
            let state = UpdateState {
                last_check: Some(now()),
                last_available: Some("1.2.3".to_owned())
            };

            let toml = toml::to_string(&state).unwrap();

            assert_eq!(
                toml,
                "last_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\"\n"
            );
        }

        #[test]
        fn from_str_deserializes_from_toml()
        {
            let state: UpdateState = toml::from_str(
                "last_check = \"2026-06-06T12:00:00Z\"\nlast_available = \"1.2.3\""
            )
            .unwrap();

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
            let due = state.is_due(
                Frequency::Every(StdDuration::from_secs(60 * 60)),
                now()
            );

            // Assert
            assert!(!due);
        }
    }
}
