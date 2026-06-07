use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency
{
    Every(Duration),
    Never
}

impl Frequency
{
    pub fn from_days(days: u64) -> Self
    {
        Self::Every(Duration::from_secs(60 * 60 * 24 * days))
    }

    pub fn interval(self) -> Option<Duration>
    {
        match self
        {
            Self::Every(duration) => Some(duration),
            Self::Never => None
        }
    }
}

impl FromStr for Frequency
{
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self>
    {
        match value
        {
            "never" => Ok(Self::Never),

            _ => humantime::parse_duration(value).map(Self::Every).with_context(
                || format!("unsupported update check frequency '{value}'")
            )
        }
    }
}

impl Display for Frequency
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult
    {
        match self
        {
            Self::Every(duration) =>
                f.write_str(&humantime::format_duration(*duration).to_string()),

            Self::Never => f.write_str("never")
        }
    }
}

impl Serialize for Frequency
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Frequency
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn parse_supported_duration_frequencies()
    {
        // Arrange
        let cases = [
            ("1h", Frequency::Every(Duration::from_secs(60 * 60))),
            ("1 day", Frequency::Every(Duration::from_secs(60 * 60 * 24))),
            ("7 days", Frequency::Every(Duration::from_secs(60 * 60 * 24 * 7))),
            (
                "30 days",
                Frequency::Every(Duration::from_secs(60 * 60 * 24 * 30))
            ),
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
        let err = "daily".parse::<Frequency>().unwrap_err();

        // Assert
        assert!(err.to_string().contains("unsupported update check frequency"));
    }

    #[test]
    fn display_formats_duration_with_humantime()
    {
        // Arrange
        let frequency =
            Frequency::Every(Duration::from_secs(60 * 60 * 24 * 30));

        // Act
        let value = frequency.to_string();

        // Assert
        assert_eq!(value, "30days");
    }
}
