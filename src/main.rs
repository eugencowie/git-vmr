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
    let result = cli.run();

    // Handle errors
    if let Err(error) = result
    {
        if let Some(aggregate) = error.downcast_ref::<AggregateError>()
        {
            for error in aggregate.errors()
            {
                eprintln!("{error}");
            }
        }
        else
        {
            eprintln!("fatal: {error:#}");
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
