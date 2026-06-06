use anyhow::{Context, Result, bail};
use axoasset::reqwest::Client;
use axoupdater::AxoUpdater;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration as StdDuration;
use std::{fmt, fs};

const APP_NAME: &str = "git-vmr";
const QUERY_TIMEOUT: StdDuration = StdDuration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckFrequency
{
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Never
}

impl CheckFrequency
{
    fn interval(self) -> Option<Duration>
    {
        match self
        {
            Self::Hourly => Some(Duration::hours(1)),
            Self::Daily => Some(Duration::days(1)),
            Self::Weekly => Some(Duration::days(7)),
            Self::Monthly => Some(Duration::days(30)),
            Self::Never => None
        }
    }
}

impl FromStr for CheckFrequency
{
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self>
    {
        match value
        {
            "hourly" => Ok(Self::Hourly),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "never" => Ok(Self::Never),
            _ => bail!("unsupported update check frequency '{value}'")
        }
    }
}

impl fmt::Display for CheckFrequency
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let value = match self
        {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Never => "never"
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalUpdateConfig
{
    pub updates: UpdatesConfig
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatesConfig
{
    pub check_frequency: CheckFrequency
}

impl Default for GlobalUpdateConfig
{
    fn default() -> Self
    {
        Self {
            updates: UpdatesConfig { check_frequency: CheckFrequency::Daily }
        }
    }
}

impl GlobalUpdateConfig
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
}

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

    fn is_due(&self, frequency: CheckFrequency, now: DateTime<Utc>) -> bool
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

#[derive(Debug, Clone)]
pub struct UpdateCheckPaths
{
    pub config_file: PathBuf,
    pub state_file: PathBuf
}

impl UpdateCheckPaths
{
    pub fn resolve() -> Result<Self>
    {
        Self::resolve_with(None, None)
    }

    fn resolve_with(
        config_dir: Option<PathBuf>,
        state_dir: Option<PathBuf>
    ) -> Result<Self>
    {
        let config_root = config_dir
            .or_else(|| {
                std::env::var_os("GIT_VMR_CONFIG_DIR").map(PathBuf::from)
            })
            .or_else(dirs::config_dir)
            .context("failed to resolve user config directory")?;
        let state_root = state_dir
            .or_else(|| {
                std::env::var_os("GIT_VMR_STATE_DIR").map(PathBuf::from)
            })
            .or_else(dirs::state_dir)
            .or_else(dirs::data_local_dir)
            .context("failed to resolve user state directory")?;

        Ok(Self {
            config_file: config_root.join(APP_NAME).join("config.toml"),
            state_file: state_root.join(APP_NAME).join("update.toml")
        })
    }
}

pub trait UpdateQuery
{
    fn query_new_version(&mut self) -> Result<QueryOutcome>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOutcome
{
    NewerVersion(String),
    CurrentVersion,
    Ineligible
}

pub struct AxoUpdateQuery;

impl UpdateQuery for AxoUpdateQuery
{
    fn query_new_version(&mut self) -> Result<QueryOutcome>
    {
        query_with_axoupdater()
    }
}

pub fn run_auto_update_check() -> Option<String>
{
    let paths = match UpdateCheckPaths::resolve()
    {
        Ok(paths) => paths,
        Err(err) => return Some(format!("warning: {err:#}"))
    };
    let mut query = AxoUpdateQuery;
    run_with_paths(&paths, Utc::now(), &mut query)
}

pub fn run_with_paths(
    paths: &UpdateCheckPaths,
    now: DateTime<Utc>,
    query: &mut dyn UpdateQuery
) -> Option<String>
{
    let config = match GlobalUpdateConfig::load(&paths.config_file)
    {
        Ok(config) => config,
        Err(err) => return Some(format!("warning: {err:#}"))
    };
    let mut state =
        UpdateCheckState::load(&paths.state_file).unwrap_or_default();
    if !state.is_due(config.updates.check_frequency, now)
    {
        return None;
    }

    state.last_attempted_check = Some(now);
    let _ = state.save(&paths.state_file);

    match query.query_new_version()
    {
        Ok(QueryOutcome::NewerVersion(version)) =>
        {
            state.last_available_version = Some(version.clone());
            let _ = state.save(&paths.state_file);
            Some(format!("A new git-vmr version is available: {version}"))
        }
        Ok(QueryOutcome::CurrentVersion | QueryOutcome::Ineligible)
        | Err(_) => None
    }
}

fn query_with_axoupdater() -> Result<QueryOutcome>
{
    let mut updater = AxoUpdater::new_for(APP_NAME);
    let client = Client::builder().timeout(QUERY_TIMEOUT).build()?;
    updater.set_client(client);
    updater.load_receipt()?;
    if !updater.check_receipt_is_for_this_executable()?
    {
        return Ok(QueryOutcome::Ineligible);
    }

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
    use std::cell::Cell;

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

    fn paths(tmp: &tempfile::TempDir) -> UpdateCheckPaths
    {
        UpdateCheckPaths {
            config_file: tmp
                .path()
                .join("config")
                .join(APP_NAME)
                .join("config.toml"),
            state_file: tmp
                .path()
                .join("state")
                .join(APP_NAME)
                .join("update.toml")
        }
    }

    fn now() -> DateTime<Utc>
    {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn missing_global_config_defaults_to_daily()
    {
        let tmp = tempfile::tempdir().unwrap();
        let config =
            GlobalUpdateConfig::load(&tmp.path().join("missing.toml")).unwrap();

        assert_eq!(config.updates.check_frequency, CheckFrequency::Daily);
    }

    #[test]
    fn parses_supported_frequencies()
    {
        for (value, expected) in [
            ("hourly", CheckFrequency::Hourly),
            ("daily", CheckFrequency::Daily),
            ("weekly", CheckFrequency::Weekly),
            ("monthly", CheckFrequency::Monthly),
            ("never", CheckFrequency::Never)
        ]
        {
            assert_eq!(value.parse::<CheckFrequency>().unwrap(), expected);
        }
    }

    #[test]
    fn rejects_unsupported_frequency()
    {
        let err = "yearly".parse::<CheckFrequency>().unwrap_err();

        assert!(err.to_string().contains("unsupported update check frequency"));
    }

    #[test]
    fn never_skips_update_checks()
    {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(&tmp);
        fs::create_dir_all(paths.config_file.parent().unwrap()).unwrap();
        fs::write(
            &paths.config_file,
            "[updates]\ncheck_frequency = \"never\"\n"
        )
        .unwrap();
        let mut query =
            FakeQuery::new(QueryOutcome::NewerVersion("9.0.0".into()));

        let notice = run_with_paths(&paths, now(), &mut query);

        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 0);
    }

    #[test]
    fn state_roundtrips()
    {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("state").join("update.toml");
        let state = UpdateCheckState {
            last_attempted_check: Some(now()),
            last_available_version: Some("1.2.3".to_owned())
        };

        state.save(&path).unwrap();
        let loaded = UpdateCheckState::load(&path).unwrap();

        assert_eq!(loaded, state);
    }

    #[test]
    fn missing_state_is_due()
    {
        let state = UpdateCheckState::default();

        assert!(state.is_due(CheckFrequency::Daily, now()));
    }

    #[test]
    fn recent_state_is_not_due()
    {
        let state = UpdateCheckState {
            last_attempted_check: Some(now() - Duration::hours(23)),
            last_available_version: None
        };

        assert!(!state.is_due(CheckFrequency::Daily, now()));
    }

    #[test]
    fn expired_state_is_due()
    {
        let state = UpdateCheckState {
            last_attempted_check: Some(now() - Duration::days(31)),
            last_available_version: None
        };

        assert!(state.is_due(CheckFrequency::Monthly, now()));
    }

    #[test]
    fn records_attempt_before_query()
    {
        struct FailingQuery<'a>
        {
            paths: &'a UpdateCheckPaths,
            now: DateTime<Utc>
        }

        impl UpdateQuery for FailingQuery<'_>
        {
            fn query_new_version(&mut self) -> Result<QueryOutcome>
            {
                let state =
                    UpdateCheckState::load(&self.paths.state_file).unwrap();
                assert_eq!(state.last_attempted_check, Some(self.now));
                bail!("network failed")
            }
        }

        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(&tmp);
        let mut query = FailingQuery { paths: &paths, now: now() };

        let notice = run_with_paths(&paths, now(), &mut query);

        assert_eq!(notice, None);
    }

    #[test]
    fn new_version_prints_version_only_notice()
    {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(&tmp);
        let mut query =
            FakeQuery::new(QueryOutcome::NewerVersion("1.2.3".into()));

        let notice = run_with_paths(&paths, now(), &mut query).unwrap();

        assert_eq!(notice, "A new git-vmr version is available: 1.2.3");
    }

    #[test]
    fn current_version_is_quiet()
    {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(&tmp);
        let mut query = FakeQuery::new(QueryOutcome::CurrentVersion);

        let notice = run_with_paths(&paths, now(), &mut query);

        assert_eq!(notice, None);
    }

    #[test]
    fn ineligible_receipt_is_quiet()
    {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(&tmp);
        let mut query = FakeQuery::new(QueryOutcome::Ineligible);

        let notice = run_with_paths(&paths, now(), &mut query);

        assert_eq!(notice, None);
    }

    #[test]
    fn query_failure_is_quiet_and_non_fatal()
    {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(&tmp);
        let mut query =
            FakeQuery { outcome: FakeOutcome::Err, calls: Cell::new(0) };

        let notice = run_with_paths(&paths, now(), &mut query);

        assert_eq!(notice, None);
        assert_eq!(query.calls.get(), 1);
    }

    #[test]
    fn resolves_user_level_paths_outside_gitvmr()
    {
        let tmp = tempfile::tempdir().unwrap();
        let paths = UpdateCheckPaths::resolve_with(
            Some(tmp.path().join("config")),
            Some(tmp.path().join("state"))
        )
        .unwrap();

        assert_eq!(
            paths.config_file,
            tmp.path().join("config").join(APP_NAME).join("config.toml")
        );
        assert_eq!(
            paths.state_file,
            tmp.path().join("state").join(APP_NAME).join("update.toml")
        );
    }

    #[test]
    fn version_type_from_axoupdater_is_usable()
    {
        let version: axoupdater::Version = "1.2.3".parse().unwrap();

        assert_eq!(version.to_string(), "1.2.3");
    }
}
