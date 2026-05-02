mod init;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Command
{
    /// Create an empty virtual monorepo or reinitialize an existing one
    Init
}

#[derive(Parser)]
#[command(name = "git-vmr")]
pub struct Cli
{
    #[command(subcommand)]
    command: Command
}

impl Cli
{
    pub fn run(self) -> Result<()>
    {
        let working_dir = std::env::current_dir()?;

        // Run command
        match self.command
        {
            Command::Init => init::init(&working_dir)
        }
    }
}
