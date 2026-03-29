use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::repo;

pub fn hash_bytes(data: &[u8]) -> String {
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
    Ok(repo::objects_dir().join(prefix).join(rest))
}

pub fn write_object(bytes: &[u8]) -> Result<String> {
    repo::ensure_repo_exists()?;

    let hash = hash_bytes(bytes);
    let path = object_path(&hash)?;

    if path.exists() {
        return Ok(hash);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("failed to create object subdirectory")?;
    }

    fs::write(&path, bytes).with_context(|| format!("failed to write object {hash}"))?;
    Ok(hash)
}

pub fn read_object(hash: &str) -> Result<Vec<u8>> {
    repo::ensure_repo_exists()?;

    let path = object_path(hash)?;
    let data = fs::read(&path).with_context(|| format!("object not found: {hash}"))?;
    Ok(data)
}
