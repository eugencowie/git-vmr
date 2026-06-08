use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Core
{
    /// Configuration schema version
    #[serde(default)]
    pub version: u32
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn default_has_default_values()
    {
        // Act
        let core = Core::default();

        // Assert
        assert_eq!(core.version, 0);
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        // Arrange
        let core = Core::default();

        // Act
        let toml = toml::to_string(&core).unwrap();

        // Assert
        assert_eq!(toml, "version = 0\n");
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        // Act
        let core: Core = toml::from_str("version = 42").unwrap();

        // Assert
        assert_eq!(core.version, 42);
    }

    #[test]
    fn from_str_defaults_missing_version_to_zero()
    {
        // Act
        let core: Core = toml::from_str("").unwrap();

        // Assert
        assert_eq!(core.version, 0);
    }
}
