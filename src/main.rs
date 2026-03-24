use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};

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
    Add { file: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => init_repo()?,
        Commands::Add { file } => add_file(&file)?,
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

fn ensure_repo_exists() -> Result<()> {
    if !drax_dir().is_dir()
        || !objects_dir().is_dir()
        || !refs_dir().is_dir()
        || !head_file().is_file()
        || !index_file().is_file()
    {
        bail!("drax repository not initialized. run `drax init` first");
    }
    Ok(())
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

fn object_path(hash: &str) -> Result<PathBuf> {
    if hash.len() < 3 {
        bail!("invalid hash: {hash}");
    }

    let (prefix, rest) = hash.split_at(2);
    Ok(objects_dir().join(prefix).join(rest))
}

fn write_object(bytes: &[u8]) -> Result<String> {
    ensure_repo_exists()?;

    let hash = hash_bytes(bytes);
    let path = object_path(&hash)?;

    if path.exists() {
        return Ok(hash);
    }

    /*
       static string -> context
       dynamic/path-rich message -> with_context

       so, these both are valid,
       fs::create_dir_all(parent).context("failed to create object subdirectory")?;
       fs::create_dir_all(parent).with_context(|| format!("failed to create object subdirectory: {}", parent.display()))?;
    */
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("failed to create object subdirectory")?;
    }

    fs::write(&path, bytes).with_context(|| format!("failed to write object {hash}"))?;
    Ok(hash)
}

fn read_object(hash: &str) -> Result<Vec<u8>> {
    ensure_repo_exists()?;

    let path = object_path(hash)?;
    let data = fs::read(&path).with_context(|| format!("object not found: {hash}"))?;
    Ok(data)
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

fn load_index() -> Result<BTreeMap<String, String>> {
    ensure_repo_exists()?;
    let index_path = index_file();

    if !index_path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = fs::read_to_string(&index_path).context("failed to read index")?;
    let mut entries = BTreeMap::new();

    for (i, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let (file, hash) = line
            .split_once("\t")
            .with_context(|| format!("invalid index line {}", i + 1))?;
        entries.insert(file.to_string(), hash.to_string());
    }

    Ok(entries)
}

fn save_index(entries: &BTreeMap<String, String>) -> Result<()> {
    ensure_repo_exists()?;
    let mut out = String::new();

    for (file, hash) in entries {
        out.push_str(file);
        out.push('\t');
        out.push_str(hash);
        out.push('\n');
    }

    fs::write(index_file(), out).context("failed to write index")?;
    Ok(())
}

fn add_file(file: &str) -> Result<()> {
    ensure_repo_exists()?;

    if !PathBuf::from(file).is_file() {
        bail!("not a file: {file}");
    }

    let bytes = fs::read(file).with_context(|| format!("failed to read file: {file}"))?;
    let hash = write_object(&bytes)?;

    let mut idx = load_index()?;
    idx.insert(file.replace("\\", "/"), hash.clone());
    save_index(&idx)?;

    println!("added {} {}", file, &hash[..12]);
    Ok(())
}
