mod cli;
mod commands;
mod config;
mod git;
mod state;
mod update_check;
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
        eprintln!("{e:#}");
        return ExitCode::FAILURE;
    }

    if let Some(notice) = update_check::run_auto_update_check()
    {
        eprintln!("{notice}");
    }

    ExitCode::SUCCESS
}
