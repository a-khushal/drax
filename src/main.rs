use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => init_repo()?,
    }

    Ok(())
}

fn init_repo() -> Result<()> {
    let drax_dir = PathBuf::from(".drax");
    let objects_dir = drax_dir.join("objects");
    let refs_dir = drax_dir.join("refs");
    let head_file = drax_dir.join("HEAD");
    let index_file = drax_dir.join("index");

    if drax_dir.exists() {
        bail!("repository already initialized at .drax");
    }

    fs::create_dir(&drax_dir).context("failed to create .drax")?;
    fs::create_dir_all(&objects_dir).context("failed to create objects directory")?;
    fs::create_dir_all(&refs_dir).context("failed to create refs directory")?;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&head_file)
        .context("failed to create HEAD file")?;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&index_file)
        .context("failed to create index file")?;

    println!(
        "Initialized empty drax repository in {}",
        drax_dir.display()
    );
    Ok(())
}
