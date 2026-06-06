use anyhow::{Result, bail};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frequency
{
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Never
}

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
        Self { check_frequency: Frequency::Daily }
    }
}

impl FromStr for Frequency
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

impl Display for Frequency
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult
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

impl Frequency
{
    pub fn interval(self) -> Option<Duration>
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
        assert_eq!(updates.check_frequency, Frequency::Daily);
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        // Arrange
        let updates = Updates::default();

        // Act
        let toml = toml::to_string(&updates).unwrap();

        // Assert
        assert_eq!(toml, "checkfrequency = \"daily\"\n");
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        // Act
        let updates: Updates =
            toml::from_str("checkfrequency = \"weekly\"").unwrap();

        // Assert
        assert_eq!(updates.check_frequency, Frequency::Weekly);
    }

    #[test]
    fn parse_supported_frequencies()
    {
        // Arrange
        let cases = [
            ("hourly", Frequency::Hourly),
            ("daily", Frequency::Daily),
            ("weekly", Frequency::Weekly),
            ("monthly", Frequency::Monthly),
            ("never", Frequency::Never)
        ];

        // Act
        let results = cases.iter().map(|(value, expected)| {
            (value.parse::<Frequency>().unwrap(), *expected)
        });

        // Assert
        for (value, expected) in results
        {
            assert_eq!(value, expected);
        }
    }

    #[test]
    fn parse_rejects_unsupported_frequency()
    {
        // Act
        let err = "yearly".parse::<Frequency>().unwrap_err();

        // Assert
        assert!(err.to_string().contains("unsupported update check frequency"));
    }
}
