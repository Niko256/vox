use super::index::Index;
use anyhow::{Context, Result};
use std::path::Path;

pub fn ls_files_command(stage: bool) -> Result<()> {
    let index_path = Path::new(".vcs/index");
    let mut index = Index::new();

    if index_path.exists() {
        index
            .read_from_file(index_path)
            .context("Failed to read index")?;
    }

    for entry in index.entries.values() {
        if stage {
            println!(
                "{} {} {}\t{}",
                format!("{:o}", entry.mode),
                hex::encode(&entry.hash),
                entry.flags,
                entry.path.display()
            );
        } else {
            println!("{}", entry.path.display());
        }
    }

    Ok(())
}
