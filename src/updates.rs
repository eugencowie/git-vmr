use crate::cli::{APP_NAME, CliContext};
use anyhow::Result;
use axoupdater::AxoUpdater;
use chrono::{DateTime, Utc};
use std::time::Duration as StdDuration;

const QUERY_TIMEOUT: StdDuration = StdDuration::from_secs(5);

/// Check for updates and return a notice if a new version is available
pub fn check(context: &mut CliContext) -> Option<String>
{
    check_with_query(context, Utc::now(), query_with_axoupdater)
}

/// Run a scheduled update check
fn check_with_query(
    context: &mut CliContext,
    now: DateTime<Utc>,
    mut query: impl FnMut() -> Result<Option<String>>
) -> Option<String>
{
    if !context
        .global_state
        .updates
        .is_due(context.global_config.updates.check_frequency, now)
    {
        return None;
    }

    context.global_state.updates.last_check = Some(now);
    context.global_state.mark_dirty();
    context.save().ok()?;

    let version = query().ok()??;
    context.global_state.updates.last_available = Some(version.clone());
    context.global_state.mark_dirty();

    Some(format!("A new git-vmr version is available: {version}"))
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
    use crate::config::{Core, Frequency, Updates};
    use crate::state::GlobalState;
    use anyhow::bail;
    use std::cell::Cell;
    use std::fs;
    use std::path::PathBuf;

    fn run_with_paths(
        frequency: Frequency,
        state_file: &std::path::Path,
        now: DateTime<Utc>,
        query: impl FnMut() -> Result<Option<String>>
    ) -> Option<String>
    {
        use crate::config::GlobalConfig;
        use std::path::PathBuf;

        let state = crate::state::GlobalState::load_from_path(state_file);
        let mut context = CliContext {
            display_name: "git vmr".to_owned(),
            working_dir: PathBuf::new(),
            global_config: GlobalConfig {
                core: Core::default(),
                updates: Updates { check_frequency: frequency }
            },
            global_state: state
        };
        let notice = check_with_query(&mut context, now, query);
        context.save().ok()?;
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
            run_with_paths(Frequency::Never, &state_file, now(), || {
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
        let notice = run_with_paths(daily(), &state_file, now(), || {
            let state = GlobalState::load_from_path(&state_file);
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
        let notice = run_with_paths(daily(), &state_file, now(), || {
            calls.set(calls.get() + 1);
            Ok(Some("1.2.3".into()))
        });

        // Assert
        assert_eq!(
            notice,
            Some("A new git-vmr version is available: 1.2.3".to_owned())
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(
            GlobalState::load_from_path(&state_file).updates.last_available,
            Some("1.2.3".to_owned())
        );
    }

    #[test]
    fn attempt_save_failure_is_quiet_and_skips_query()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        fs::create_dir_all(state_file.parent().unwrap()).unwrap();
        fs::write(&state_file, "").unwrap();
        let original_permissions =
            fs::metadata(&state_file).unwrap().permissions();
        let mut readonly_permissions = original_permissions.clone();
        readonly_permissions.set_readonly(true);
        fs::set_permissions(&state_file, readonly_permissions).unwrap();
        let calls = Cell::new(0);

        // Act
        let notice = run_with_paths(daily(), &state_file, now(), || {
            calls.set(calls.get() + 1);
            Ok(Some("1.2.3".into()))
        });

        // Cleanup
        fs::set_permissions(&state_file, original_permissions).unwrap();

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
        let notice = run_with_paths(daily(), &state_file, now(), || {
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
        let notice = run_with_paths(daily(), &state_file, now(), || {
            Ok(Some("1.2.3".into()))
        })
        .unwrap();

        // Assert
        assert_eq!(notice, "A new git-vmr version is available: 1.2.3");
    }

    #[test]
    fn current_version_is_quiet()
    {
        // Arrange
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);

        // Act
        let notice = run_with_paths(daily(), &state_file, now(), || Ok(None));

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
        let notice = run_with_paths(daily(), &state_file, now(), || {
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
}
