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

use cli::{Cli, SilentError};
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Run command and handle errors
    if let Err(e) = cli.run()
    {
        // A silent error should not be displayed
        if !e.is::<SilentError>()
        {
            anstream::eprintln!("{e:#}");
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
