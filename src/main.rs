mod cli;
mod config;
mod git;
mod vmr;

use cli::{AggregateError, Cli};
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Run command
    if let Err(e) = cli.run()
    {
        if let Some(aggregate) = e.downcast_ref::<AggregateError>()
        {
            for error in aggregate.errors()
            {
                eprintln!("{}", fatal_message(error));
            }
        }
        else
        {
            eprintln!("fatal: {e:#}");
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn fatal_message(message: &str) -> String
{
    if message.starts_with("fatal:")
    {
        message.to_owned()
    }
    else
    {
        format!("fatal: {message}")
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn fatal_message_does_not_duplicate_existing_prefix()
    {
        assert_eq!(
            fatal_message("fatal: invalid reference: feature/auth"),
            "fatal: invalid reference: feature/auth"
        );
    }

    #[test]
    fn fatal_message_adds_missing_prefix()
    {
        assert_eq!(
            fatal_message("invalid reference"),
            "fatal: invalid reference"
        );
    }
}
