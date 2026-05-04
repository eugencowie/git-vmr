mod core;
pub mod vmr;

use core::Core;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config
{
    pub core: Core
}

impl Default for Config
{
    fn default() -> Self
    {
        Self { core: Core { version: 0 } }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn default_has_version_zero()
    {
        // Act
        let config = Config::default();

        // Assert
        assert_eq!(config.core.version, 0);
    }

    #[test]
    fn serializes_to_toml()
    {
        // Arrange
        let config = Config::default();

        // Act
        let toml = toml::to_string(&config).unwrap();

        // Assert
        assert!(toml.contains("[core]"));
        assert!(toml.contains("version = 0"));
    }

    #[test]
    fn deserializes_from_toml()
    {
        // Act
        let config: Config = toml::from_str("[core]\nversion = 5").unwrap();

        // Assert
        assert_eq!(config.core.version, 5);
    }

    #[test]
    fn roundtrip()
    {
        // Arrange
        let original = Config::default();
        let toml = toml::to_string(&original).unwrap();

        // Act
        let parsed: Config = toml::from_str(&toml).unwrap();

        // Assert
        assert_eq!(parsed, original);
    }
}
