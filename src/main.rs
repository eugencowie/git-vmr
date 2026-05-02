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
        eprintln!("fatal: {e:#}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
