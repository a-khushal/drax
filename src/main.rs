use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

const DRAX_DIR: &str = ".drax";
const OBJECTS_DIR: &str = ".drax/objects";
const REFS_DIR: &str = ".drax/refs";
const HEAD_FILE: &str = ".drax/HEAD";
const INDEX_FILE: &str = ".drax/index";

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

fn drax_dir() -> PathBuf {
    PathBuf::from(DRAX_DIR)
}

fn objects_dir() -> PathBuf {
    PathBuf::from(OBJECTS_DIR)
}

fn refs_dir() -> PathBuf {
    PathBuf::from(REFS_DIR)
}

fn head_file() -> PathBuf {
    PathBuf::from(HEAD_FILE)
}

fn index_file() -> PathBuf {
    PathBuf::from(INDEX_FILE)
}

fn ensure_repo_not_initialized() -> Result<()> {
    if drax_dir().exists() {
        bail!("repository already initialized at {DRAX_DIR}");
    }

    Ok(())
}

fn init_repo() -> Result<()> {
    ensure_repo_not_initialized()?;

    let drax_dir = drax_dir();
    let objects_dir = objects_dir();
    let refs_dir = refs_dir();
    let head_file = head_file();
    let index_file = index_file();

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
