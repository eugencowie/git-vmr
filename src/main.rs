mod analytics;
mod cli;
mod commands;
mod config;
mod git;
mod state;
mod updates;
mod vmr;

use cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Run command and handle errors
    if let Err(e) = cli.run()
    {
        // A silent exit means git already reported the failure on stderr
        if e.downcast_ref::<commands::clone::SilentExit>().is_none()
        {
            eprintln!("{e:#}");
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
