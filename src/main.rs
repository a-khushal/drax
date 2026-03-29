mod cli;
mod commit;
mod index;
mod object_store;
mod repo;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => repo::init_repo()?,
        Commands::Add { file } => index::add_file(&file)?,
        Commands::Commit { msg } => commit::commit(&msg)?,
        Commands::Log => commit::log_history()?,
    }

    Ok(())
}
