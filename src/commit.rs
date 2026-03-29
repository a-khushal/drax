use anyhow::{Context, Result, bail};

use crate::{index, object_store, repo};

pub fn build_tree(entries: &index::IndexEntries) -> Result<String> {
    let mut lines = Vec::new();

    for (path, hash) in entries {
        lines.push(format!("{}\t{}\tblob", path, hash));
    }

    lines.sort();
    let content = lines.join("\n");
    object_store::write_object(content.as_bytes())
}

pub fn commit(msg: &str) -> Result<()> {
    repo::ensure_repo_exists()?;

    let idx = index::load_index()?;
    if idx.is_empty() {
        bail!("nothing staged to commit");
    }

    let tree_hash = build_tree(&idx)?;
    let parent = repo::read_head()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let commit_content = format!(
        "tree {}\nparent {}\nauthor drax <drax@local>\ntimestamp {}\n\n{}\n",
        tree_hash,
        parent.as_deref().unwrap_or(""),
        timestamp,
        msg
    );

    let commit_hash = object_store::write_object(commit_content.as_bytes())?;
    repo::write_head(&commit_hash)?;

    println!("committed {} {}", &commit_hash[..12], msg);
    Ok(())
}

struct CommitView {
    parent: Option<String>,
    timestamp: i64,
    message: String,
}

fn parse_commit(bytes: &[u8], hash: &str) -> Result<CommitView> {
    let text =
        std::str::from_utf8(bytes).with_context(|| format!("commit object is not utf8: {hash}"))?;
    let (headers, message_block) = text.split_once("\n\n").unwrap_or((text, ""));

    let mut parent = None;
    let mut timestamp = None;

    for line in headers.lines() {
        if let Some(v) = line.strip_prefix("parent ") {
            let v = v.trim();
            if !v.is_empty() {
                parent = Some(v.to_string());
            }
        } else if let Some(v) = line.strip_prefix("timestamp ") {
            timestamp = Some(
                v.trim()
                    .parse::<i64>()
                    .with_context(|| format!("invalid timestamp in commit {hash}"))?,
            );
        }
    }

    let timestamp = timestamp.with_context(|| format!("missing timestamp in commit {hash}"))?;
    let message = message_block.trim_end().to_string();

    Ok(CommitView {
        parent,
        timestamp,
        message,
    })
}

pub fn log_history() -> Result<()> {
    repo::ensure_repo_exists()?;

    let mut current = repo::read_head()?;
    if current.is_none() {
        println!("no commits yet");
        return Ok(());
    }

    while let Some(hash) = current {
        let bytes = object_store::read_object(&hash)?;
        let commit = parse_commit(&bytes, &hash)?;

        println!("commit {}", hash);
        println!("timestamp {}", commit.timestamp);
        println!();
        println!("{}", commit.message);
        println!();

        current = commit.parent;
    }

    Ok(())
}
