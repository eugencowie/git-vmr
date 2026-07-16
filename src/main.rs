mod analytics;
mod cli;
mod commands;
mod config;
mod git;
mod render;
mod state;
mod store;
#[cfg(test)]
mod test_support;
mod updates;
mod vmr;
mod workspace;

use anstream::eprintln;
use cli::{Cli, SilentError};
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Run command
    if let Err(e) = cli.run()
    {
        // Suppress silent errors
        if !e.is::<SilentError>()
        {
            eprintln!("{e:#}");
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
