use super::index::Index;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

pub fn rm_command(paths: &[PathBuf], cached: bool, force: bool) -> Result<()> {
    let index_path = Path::new(".vcs/index");
    let mut index = Index::new();

    if index_path.exists() {
        index.read_from_file(index_path)?;
    }

    for path in paths {
        let normalized_path = if path.is_absolute() {
            path.clone()
        } else {
            let path_str = path.to_string_lossy();
            if path_str.starts_with("./") {
                path.clone()
            } else {
                PathBuf::from(".").join(path)
            }
        };

        let index_entry = match index.get_entry(&normalized_path) {
            Some(entry) => entry,
            None => {
                return Err(anyhow::anyhow!(
                    "Path '{}' not found in index",
                    normalized_path.display()
                ));
            }
        };

        if !force && normalized_path.exists() {
            let metadata = fs::metadata(&normalized_path).with_context(|| {
                format!("Failed to get metadata for {}", normalized_path.display())
            })?;

            if metadata.size() as u32 != index_entry.size
                || metadata.mtime() as u64 != index_entry.mtime
            {
                return Err(anyhow::anyhow!(
                    "File '{}' has local modifications. Use --force to remove anyway",
                    normalized_path.display()
                ));
            }
        }

        if !cached {
            if normalized_path.exists() {
                if normalized_path.is_dir() {
                    fs::remove_dir_all(&normalized_path)?;
                } else {
                    fs::remove_file(&normalized_path)?;
                }
            }
        }

        index.remove_entry(&normalized_path);
    }

    index.write_to_file(index_path)?;
    Ok(())
}
