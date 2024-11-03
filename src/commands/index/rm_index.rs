use super::index::Index;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn rm_command(path: &Path, cached: bool) -> Result<()> {
    let index_path = Path::new(".vcs/index");
    let mut index = Index::new();

    if index_path.exists() {
        index
            .read_from_file(index_path)
            .context("Failed to read index")?;
    }

    if index.get_entry(path).is_none() {
        return Err(anyhow::anyhow!(
            "Path '{}' not found in index",
            path.display()
        ));
    }

    if !cached {
        if path.exists() {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove file {}", path.display()))?;
        }
    }

    index.remove_entry(path);

    index
        .write_to_file(index_path)
        .context("Failed to write index")?;

    Ok(())
}
