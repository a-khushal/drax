use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};

pub const DRAX_DIR: &str = ".drax";
pub const OBJECTS_DIR: &str = ".drax/objects";
pub const REFS_DIR: &str = ".drax/refs";
pub const HEAD_FILE: &str = ".drax/HEAD";
pub const INDEX_FILE: &str = ".drax/index";

pub fn drax_dir() -> PathBuf {
    PathBuf::from(DRAX_DIR)
}

pub fn objects_dir() -> PathBuf {
    PathBuf::from(OBJECTS_DIR)
}

pub fn refs_dir() -> PathBuf {
    PathBuf::from(REFS_DIR)
}

pub fn head_file() -> PathBuf {
    PathBuf::from(HEAD_FILE)
}

pub fn index_file() -> PathBuf {
    PathBuf::from(INDEX_FILE)
}

pub fn ensure_repo_not_initialized() -> Result<()> {
    if drax_dir().exists() {
        bail!("repository already initialized at {DRAX_DIR}");
    }

    Ok(())
}

pub fn ensure_repo_exists() -> Result<()> {
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

pub fn init_repo() -> Result<()> {
    ensure_repo_not_initialized()?;

    fs::create_dir(drax_dir()).context("failed to create .drax")?;
    fs::create_dir_all(objects_dir()).context("failed to create objects directory")?;
    fs::create_dir_all(refs_dir()).context("failed to create refs directory")?;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(head_file())
        .context("failed to create HEAD file")?;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(index_file())
        .context("failed to create index file")?;

    println!(
        "Initialized empty drax repository in {}",
        drax_dir().display()
    );

    Ok(())
}

pub fn read_head() -> Result<Option<String>> {
    let path = head_file();
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let hash = content.trim();
    if hash.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hash.to_string()))
    }
}

pub fn write_head(hash: &str) -> Result<()> {
    fs::write(head_file(), hash).context("failed to write HEAD")?;
    Ok(())
}
