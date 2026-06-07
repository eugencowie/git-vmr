use crate::config::Frequency;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn default_has_default_values()
    {
        // Act
        let updates = Updates::default();

        // Assert
        assert_eq!(updates.check_frequency, Frequency::from_days(1));
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        // Arrange
        let updates = Updates::default();

        // Act
        let toml = toml::to_string(&updates).unwrap();

        // Assert
        assert_eq!(toml, "checkfrequency = \"1day\"\n");
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        // Act
        let updates: Updates =
            toml::from_str("checkfrequency = \"1 week\"").unwrap();

        // Assert
        assert_eq!(updates.check_frequency, Frequency::from_days(7));
    }
}
