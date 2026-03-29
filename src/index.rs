use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{object_store, repo};

pub type IndexEntries = BTreeMap<String, String>;

pub fn load_index() -> Result<IndexEntries> {
    repo::ensure_repo_exists()?;
    let index_path = repo::index_file();

    if !index_path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = fs::read_to_string(index_path).context("failed to read index")?;
    let mut entries = BTreeMap::new();

    for (i, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let (file, hash) = line
            .split_once('\t')
            .with_context(|| format!("invalid index line {}", i + 1))?;
        entries.insert(file.to_string(), hash.to_string());
    }

    Ok(entries)
}

pub fn save_index(entries: &IndexEntries) -> Result<()> {
    repo::ensure_repo_exists()?;
    let mut out = String::new();

    for (file, hash) in entries {
        out.push_str(file);
        out.push('\t');
        out.push_str(hash);
        out.push('\n');
    }

    fs::write(repo::index_file(), out).context("failed to write index")?;
    Ok(())
}

fn normalized_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn validate_file(path: &str) -> Result<()> {
    let file_path = PathBuf::from(path);
    if !file_path.is_file() {
        bail!("not a file: {path}");
    }

    if path_starts_with_drax_dir(&file_path) {
        bail!("cannot add files from .drax directory");
    }

    Ok(())
}

fn path_starts_with_drax_dir(path: &Path) -> bool {
    let repo_dir = Path::new(repo::DRAX_DIR);
    path.starts_with(repo_dir)
}

pub fn add_file(file: &str) -> Result<()> {
    repo::ensure_repo_exists()?;
    validate_file(file)?;

    let bytes = fs::read(file).with_context(|| format!("failed to read file: {file}"))?;
    let hash = object_store::write_object(&bytes)?;

    let mut idx = load_index()?;
    idx.insert(normalized_path(file), hash.clone());
    save_index(&idx)?;

    println!("added {} {}", file, &hash[..12]);
    Ok(())
}
