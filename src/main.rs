mod cli;
mod commands;
mod config;
mod git;
mod state;
mod update_check;
mod vmr;

use cli::Cli;
use config::GlobalConfig;
use std::process::ExitCode;

fn main() -> ExitCode
{
    // Parse arguments
    let cli = Cli::parse();

    // Load global config
    let global_config = match GlobalConfig::load_global()
    {
        Ok(config) => config,
        Err(e) =>
        {
            eprintln!("{e:#}");
            return ExitCode::FAILURE;
        }
    };

    // Run command and handle errors
    if let Err(e) = cli.run()
    {
        eprintln!("{e:#}");
        return ExitCode::FAILURE;
    }

    if let Some(notice) = update_check::run_auto_update_check(&global_config)
    {
        eprintln!("{notice}");
    }

    ExitCode::SUCCESS
}
