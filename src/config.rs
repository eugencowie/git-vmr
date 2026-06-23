mod core;
mod updates;

use crate::cli::APP_NAME;
use anyhow::{Context, Result};
pub use core::Core;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs};
pub use updates::{Frequency, Updates};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config
{
    /// Core configuration
    pub core: Core
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalConfig
{
    /// Core configuration
    pub core: Core,

    /// Software updates
    pub updates: Updates
}

impl GlobalConfig
{
    /// Load global configuration
    pub fn load() -> Result<Self>
    {
        // Resolve config path
        let config_path = Self::resolve_path(None)?;

        // Load config from path
        Self::load_from_path(&config_path)
    }

    /// Load global configuration from path
    fn load_from_path(path: &Path) -> Result<Self>
    {
        // Read and parse config file
        match fs::read_to_string(path)
        {
            Ok(contents) => toml::from_str(&contents)
                .with_context(|| format!("failed to parse {}", path.display())),

            // Missing global config uses defaults
            Err(err) if err.kind() == ErrorKind::NotFound =>
                Ok(Self::default()),

            Err(err) => Err(err)
                .with_context(|| format!("failed to read {}", path.display()))
        }
    }

    /// Resolve global configuration file path
    fn resolve_path(config_dir: Option<PathBuf>) -> Result<PathBuf>
    {
        // Resolve config root directory
        let config_root = config_dir
            .or_else(|| env::var_os("GITVMR_CONFIG_DIR").map(PathBuf::from))
            .or_else(dirs::config_dir)
            .context("failed to resolve user config directory")?;

        // Build app-specific config path
        Ok(config_root.join(APP_NAME).join("config.toml"))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    mod config
    {
        use super::*;

        #[test]
        fn default_has_default_values()
        {
            // Act
            let config = Config::default();

            // Assert
            assert_eq!(config.core, Core::default());
        }

        #[test]
        fn to_string_serializes_to_toml()
        {
            // Arrange
            let config = Config::default();

            // Act
            let toml = toml::to_string(&config).unwrap();

            // Assert
            assert_eq!(toml, "[core]\nversion = 0\n");
        }

        #[test]
        fn from_str_deserializes_from_toml()
        {
            // Act
            let config: Config =
                toml::from_str("[core]\nversion = 42").unwrap();

            // Assert
            assert_eq!(config.core.version, 42);
        }

        #[test]
        fn from_str_defaults_missing_core_version_to_zero()
        {
            // Act
            let config: Config = toml::from_str("[core]\n").unwrap();

            // Assert
            assert_eq!(config.core.version, 0);
        }

        #[test]
        fn from_str_defaults_missing_core_section()
        {
            // Act
            let config: Config = toml::from_str("").unwrap();

            // Assert
            assert_eq!(config.core, Core::default());
        }
    }

    mod global_config
    {
        use super::*;
        use updates::Frequency;

        #[test]
        fn default_has_default_values()
        {
            // Act
            let config = GlobalConfig::default();

            // Assert
            assert_eq!(config.core, Core::default());
            assert_eq!(config.updates, Updates::default());
        }

        #[test]
        fn to_string_serializes_to_toml()
        {
            // Arrange
            let config = GlobalConfig::default();

            // Act
            let toml = toml::to_string(&config).unwrap();

            // Assert
            assert_eq!(
                toml,
                "[core]\nversion = 0\n\n[updates]\ncheckfrequency = \"1day\"\n"
            );
        }

        #[test]
        fn from_str_deserializes_from_toml()
        {
            // Act
            let config: GlobalConfig = toml::from_str(
                "[core]\nversion = 42\n\n[updates]\ncheckfrequency = \"1 week\""
            )
            .unwrap();

            // Assert
            assert_eq!(config.core.version, 42);
            assert_eq!(config.updates.check_frequency, Frequency::from_days(7));
        }

        #[test]
        fn from_str_defaults_missing_core_version_to_zero()
        {
            // Act
            let config: GlobalConfig = toml::from_str("[core]").unwrap();

            // Assert
            assert_eq!(config.core.version, 0);
            assert_eq!(config.updates.check_frequency, Frequency::from_days(1));
        }

        #[test]
        fn from_str_defaults_missing_core_section()
        {
            // Act
            let config: GlobalConfig =
                toml::from_str("[updates]\ncheckfrequency = \"1 week\"")
                    .unwrap();

            // Assert
            assert_eq!(config.core, Core::default());
            assert_eq!(config.updates.check_frequency, Frequency::from_days(7));
        }

        #[test]
        fn load_missing_file_returns_default()
        {
            // Arrange
            let tmp = tempfile::tempdir().unwrap();

            // Act
            let config =
                GlobalConfig::load_from_path(&tmp.path().join("missing.toml"))
                    .unwrap();

            // Assert
            assert_eq!(config, GlobalConfig::default());
        }

        #[test]
        fn load_rejects_malformed_file()
        {
            // Arrange
            let tmp = tempfile::tempdir().unwrap();
            let path = tmp.path().join("config.toml");
            fs::write(&path, "[updates]\ncheckfrequency = \"daily\"\n")
                .unwrap();

            // Act
            let err = GlobalConfig::load_from_path(&path).unwrap_err();

            // Assert
            assert!(err.to_string().contains("failed to parse"));
        }

        #[test]
        fn path_uses_user_level_config_file()
        {
            // Arrange
            let tmp = tempfile::tempdir().unwrap();

            // Act
            let path =
                GlobalConfig::resolve_path(Some(tmp.path().join("config")))
                    .unwrap();

            // Assert
            assert_eq!(
                path,
                tmp.path().join("config").join("git-vmr").join("config.toml")
            );
        }
    }
}
