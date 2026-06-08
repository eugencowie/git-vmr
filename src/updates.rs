use crate::cli::{APP_NAME, CliContext};
use crate::config::{Frequency, GlobalConfig};
use crate::state::GlobalState;
use anyhow::Result;
use axoasset::reqwest::Client;
use axoupdater::AxoUpdater;
use chrono::{DateTime, Utc};
use std::time::Duration as StdDuration;

const QUERY_TIMEOUT: StdDuration = StdDuration::from_secs(5);

pub trait UpdateQuery
{
    fn query_new_version(&mut self) -> Result<QueryOutcome>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOutcome
{
    NewerVersion(String),
    CurrentVersion
}

pub struct AxoUpdateQuery;

impl UpdateQuery for AxoUpdateQuery
{
    fn query_new_version(&mut self) -> Result<QueryOutcome>
    {
        query_with_axoupdater()
    }
}

pub fn check_for_updates(context: &mut CliContext) -> Option<String>
{
    if context.global_config.updates.check_frequency == Frequency::Never
    {
        return None;
    }

    let mut query = AxoUpdateQuery;
    run(
        &context.global_config,
        &mut context.global_state,
        Utc::now(),
        &mut query
    )
}

#[cfg(test)]
fn run_with_paths(
    config: &GlobalConfig,
    state_file: &std::path::Path,
    now: DateTime<Utc>,
    query: &mut dyn UpdateQuery
) -> Option<String>
{
    let mut state = match GlobalState::load_from_path(state_file)
    {
        Ok(state) => state,
        Err(_) => return None
    };
    run(config, &mut state, now, query)
}

fn run(
    config: &GlobalConfig,
    state: &mut GlobalState,
    now: DateTime<Utc>,
    query: &mut dyn UpdateQuery
) -> Option<String>
{
    if !state.updates.is_due(config.updates.check_frequency, now)
    {
        return None;
    }

    state.updates.last_check = Some(now);
    if state.save().is_err()
    {
        return None;
    }

    match query.query_new_version()
    {
        Ok(QueryOutcome::NewerVersion(version)) =>
        {
            state.updates.last_available = Some(version.clone());
            if state.save().is_err()
            {
                return None;
            }
            Some(format!("A new git-vmr version is available: {version}"))
        }
        Ok(QueryOutcome::CurrentVersion) | Err(_) => None
    }
}

fn query_with_axoupdater() -> Result<QueryOutcome>
{
    let mut updater = AxoUpdater::new_for(APP_NAME);
    let client = Client::builder().timeout(QUERY_TIMEOUT).build()?;
    updater.set_client(client);
    updater.load_receipt()?;

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

    Ok(match latest
    {
        Some(version) => QueryOutcome::NewerVersion(version.to_string()),
        None => QueryOutcome::CurrentVersion
    })
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::config::{Frequency, Updates};
    use anyhow::bail;
    use std::cell::Cell;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug)]
    struct FakeQuery
    {
        outcome: FakeOutcome,
        calls: Cell<usize>
    }

    #[derive(Debug)]
    enum FakeOutcome
    {
        Ok(QueryOutcome),
        Err
    }

    impl FakeQuery
    {
        fn new(outcome: QueryOutcome) -> Self
        {
            Self { outcome: FakeOutcome::Ok(outcome), calls: Cell::new(0) }
        }
    }

    impl UpdateQuery for FakeQuery
    {
        fn query_new_version(&mut self) -> Result<QueryOutcome>
        {
            self.calls.set(self.calls.get() + 1);
            match &self.outcome
            {
                FakeOutcome::Ok(outcome) => Ok(outcome.clone()),
                FakeOutcome::Err => Err(anyhow::anyhow!("timeout"))
            }
        }
    }

    fn state_file(tmp: &tempfile::TempDir) -> PathBuf
    {
        tmp.path().join("state").join(APP_NAME).join("state.toml")
    }

    fn config(check_frequency: Frequency) -> GlobalConfig
    {
        GlobalConfig {
            updates: Updates { check_frequency },
            ..GlobalConfig::default()
        }
    }

    fn now() -> DateTime<Utc>
    {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn never_skips_update_checks()
    {
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let config = config(Frequency::Never);
        let mut query =
            FakeQuery::new(QueryOutcome::NewerVersion("9.0.0".into()));

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 0);
    }

    #[test]
    fn records_attempt_before_query()
    {
        struct FailingQuery<'a>
        {
            state_file: &'a Path,
            now: DateTime<Utc>
        }

        impl UpdateQuery for FailingQuery<'_>
        {
            fn query_new_version(&mut self) -> Result<QueryOutcome>
            {
                let state =
                    GlobalState::load_from_path(self.state_file).unwrap();
                assert_eq!(state.updates.last_check, Some(self.now));
                bail!("network failed")
            }
        }

        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let config = GlobalConfig::default();
        let mut query = FailingQuery { state_file: &state_file, now: now() };

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        assert_eq!(notice, None);
    }

    #[test]
    fn state_load_failure_is_quiet_and_skips_query()
    {
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        fs::create_dir_all(state_file.parent().unwrap()).unwrap();
        fs::write(&state_file, "[updates]\nlast_check =").unwrap();
        let config = GlobalConfig::default();
        let mut query =
            FakeQuery::new(QueryOutcome::NewerVersion("1.2.3".into()));

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 0);
    }

    #[test]
    fn attempt_save_failure_is_quiet_and_skips_query()
    {
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        fs::create_dir_all(state_file.parent().unwrap()).unwrap();
        fs::write(&state_file, "").unwrap();
        let original_permissions =
            fs::metadata(&state_file).unwrap().permissions();
        let mut readonly_permissions = original_permissions.clone();
        readonly_permissions.set_readonly(true);
        fs::set_permissions(&state_file, readonly_permissions).unwrap();
        let config = GlobalConfig::default();
        let mut query =
            FakeQuery::new(QueryOutcome::NewerVersion("1.2.3".into()));

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        fs::set_permissions(&state_file, original_permissions).unwrap();
        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 0);
    }

    #[test]
    fn available_version_save_failure_is_quiet()
    {
        struct BlockingQuery<'a>
        {
            state_file: &'a Path,
            calls: Cell<usize>
        }

        impl UpdateQuery for BlockingQuery<'_>
        {
            fn query_new_version(&mut self) -> Result<QueryOutcome>
            {
                self.calls.set(self.calls.get() + 1);
                fs::remove_file(self.state_file).unwrap();
                fs::create_dir(self.state_file).unwrap();
                Ok(QueryOutcome::NewerVersion("1.2.3".into()))
            }
        }

        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let config = GlobalConfig::default();
        let mut query =
            BlockingQuery { state_file: &state_file, calls: Cell::new(0) };

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 1);
    }

    #[test]
    fn new_version_prints_version_only_notice()
    {
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let config = GlobalConfig::default();
        let mut query =
            FakeQuery::new(QueryOutcome::NewerVersion("1.2.3".into()));

        let notice =
            run_with_paths(&config, &state_file, now(), &mut query).unwrap();

        assert_eq!(notice, "A new git-vmr version is available: 1.2.3");
    }

    #[test]
    fn current_version_is_quiet()
    {
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let config = GlobalConfig::default();
        let mut query = FakeQuery::new(QueryOutcome::CurrentVersion);

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        assert_eq!(notice, None);
    }

    #[test]
    fn query_failure_is_quiet_and_non_fatal()
    {
        let tmp = tempfile::tempdir().unwrap();
        let state_file = state_file(&tmp);
        let config = GlobalConfig::default();
        let mut query =
            FakeQuery { outcome: FakeOutcome::Err, calls: Cell::new(0) };

        let notice = run_with_paths(&config, &state_file, now(), &mut query);

        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 1);
    }

    #[test]
    fn version_type_from_axoupdater_is_usable()
    {
        let version: axoupdater::Version = "1.2.3".parse().unwrap();

        assert_eq!(version.to_string(), "1.2.3");
    }
}
