use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use std::time::Duration;

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const MIN_FREQUENCY: Duration = Duration::from_secs(HOUR);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency
{
    /// After a given interval
    Every(Duration),

    /// Never
    Never
}

impl Frequency
{
    /// Build frequency from days
    pub fn from_days(days: u64) -> Self
    {
        Self::Every(Duration::from_secs(days * DAY))
    }

    /// Get interval if set, otherwise `None`
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

    /// Parse frequency
    fn from_str(value: &str) -> Result<Self>
    {
        match value
        {
            "never" => Ok(Self::Never),

            _ =>
            {
                // Parse human-readable duration
                let duration =
                    humantime::parse_duration(value).with_context(|| {
                        format!("unsupported frequency '{value}'")
                    })?;

                // Reject intervals that are too frequent
                if duration < MIN_FREQUENCY
                {
                    bail!("unsupported frequency '{value}'");
                }

                Ok(Self::Every(duration))
            }
        }
    }
}

impl Display for Frequency
{
    /// Format frequency
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
    /// Serialize frequency as a string
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Frequency
{
    /// Deserialize frequency from a string
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        // Parse string value with frequency validation
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    const WEEK: u64 = 7 * DAY;
    const MONTH: u64 = 2630016; // approximation used by humantime

    #[test]
    fn parse_supported_duration_frequencies()
    {
        // Arrange
        let cases = [
            ("1 hour", Frequency::Every(Duration::from_secs(HOUR))),
            ("1 day", Frequency::Every(Duration::from_secs(DAY))),
            ("1 week", Frequency::Every(Duration::from_secs(WEEK))),
            ("1 month", Frequency::Every(Duration::from_secs(MONTH))),
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
        assert!(err.to_string().contains("unsupported frequency"));
    }

    #[test]
    fn parse_rejects_frequencies_below_one_hour()
    {
        // Arrange
        let cases = ["0s", "30 minutes", "59m"];

        // Act
        let errors =
            cases.iter().map(|value| value.parse::<Frequency>().unwrap_err());

        // Assert
        for err in errors
        {
            assert!(err.to_string().contains("unsupported frequency"));
        }
    }

    #[test]
    fn display_formats_duration_with_humantime()
    {
        // Arrange
        let frequency = Frequency::Every(Duration::from_secs(MONTH));

        // Act
        let value = frequency.to_string();

        // Assert
        assert_eq!(value, "1month");
    }
}
