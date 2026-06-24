use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Analytics
{
    /// Whether usage analytics are enabled
    enabled: Option<bool>
}

impl Analytics
{
    /// Whether analytics are enabled
    pub fn enabled(&self) -> bool
    {
        self.enabled_for_major_version(env!("CARGO_PKG_VERSION_MAJOR"))
    }

    /// Whether analytics are enabled for a major version
    fn enabled_for_major_version(&self, major_version: &str) -> bool
    {
        self.enabled.unwrap_or(major_version == "0")
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn default_has_default_values()
    {
        let analytics = Analytics::default();

        assert_eq!(analytics.enabled, None);
    }

    #[test]
    fn missing_enabled_follows_version_default()
    {
        let analytics = Analytics::default();

        assert!(analytics.enabled_for_major_version("0"));
        assert!(!analytics.enabled_for_major_version("1"));
    }

    #[test]
    fn explicit_enabled_overrides_version_default()
    {
        assert!(
            !Analytics { enabled: Some(false) }.enabled_for_major_version("0")
        );
        assert!(
            Analytics { enabled: Some(true) }.enabled_for_major_version("1")
        );
    }

    #[test]
    fn to_string_serializes_to_toml()
    {
        let analytics = Analytics { enabled: Some(false) };

        let toml = toml::to_string(&analytics).unwrap();

        assert_eq!(toml, "enabled = false\n");
    }

    #[test]
    fn from_str_deserializes_from_toml()
    {
        let analytics: Analytics = toml::from_str("enabled = true").unwrap();

        assert_eq!(analytics.enabled, Some(true));
    }
}
