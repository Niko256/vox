use crate::commands::index::index::{Index, IndexEntry};
use crate::objects::blob::create_blob;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn add_command(paths: &[PathBuf], all: bool) -> Result<()> {
    let index_path = Path::new(".vcs/index");
    let mut index = Index::new();

    // Read the existing index if it exists
    if index_path.exists() {
        index.read_from_file(index_path)?;
    }

    if all {
        // Add all modified files to the index
        for entry in WalkDir::new(".")
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && !e.path().starts_with("./.vcs")
                    && !e.path().starts_with("./.git")
                    && !e.path().starts_with("./target")
            })
        {
            let path = entry.path();

            let relative_path = if path.is_absolute() {
                path.strip_prefix(std::env::current_dir()?)?
            } else {
                path
            };
            add_file_to_index(relative_path, &mut index)?;
        }
    } else {
        // Add specified files to the index
        for path in paths {
            add_file_to_index(path, &mut index)?;
        }
    }

    // Write the updated index back to the file
    index.write_to_file(index_path)?;

    Ok(())
}

fn add_file_to_index(path: &Path, index: &mut Index) -> Result<()> {
    let blob_hash = create_blob(
        path.to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
    )?;
    let hash_bytes = hex::decode(blob_hash.clone())
        .with_context(|| format!("Failed to decode blob hash: {}", blob_hash))?;

    let mut entry = IndexEntry::new(path)?;
    entry.hash.copy_from_slice(&hash_bytes);
    index.add_entry(entry);
    Ok(())
}
