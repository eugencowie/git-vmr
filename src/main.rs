mod cli;
mod config;

use clap::Parser;
use cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Run command
    if let Err(e) = cli.run()
    {
        if let Some(aggregate) = e.downcast_ref::<cli::AggregateError>()
        {
            for error in aggregate.errors()
            {
                eprintln!("fatal: {error:#}");
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
