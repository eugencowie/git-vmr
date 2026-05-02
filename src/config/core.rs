use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Core
{
    pub version: u32
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn serializes_to_toml()
    {
        // Arrange
        let core = Core { version: 0 };

        // Act
        let toml = toml::to_string(&core).unwrap();

        // Assert
        assert_eq!(toml, "version = 0\n");
    }

    #[test]
    fn deserializes_from_toml()
    {
        // Act
        let core: Core = toml::from_str("version = 42").unwrap();

        // Assert
        assert_eq!(core.version, 42);
    }
}
