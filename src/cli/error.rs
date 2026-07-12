use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// A failure that the top-level CLI must not render.
#[derive(Debug)]
pub struct SilentError;

impl Error for SilentError {}

impl Display for SilentError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        f.write_str("command failed silently")
    }
}

#[cfg(test)]
mod tests
{
    use super::SilentError;

    #[test]
    fn silent_error_has_fallback_display_text()
    {
        assert_eq!(SilentError.to_string(), "command failed silently");
    }

    #[test]
    fn silent_error_remains_detectable_when_wrapped()
    {
        let error =
            anyhow::Error::new(SilentError).context("outer command context");

        assert!(error.is::<SilentError>());
    }
}
