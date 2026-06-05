mod cli;
mod config;
mod git;
mod vmr;

use cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Run command
    let result = cli.run();

    // Handle errors
    if let Err(e) = result
    {
        eprintln!("{e:#}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
