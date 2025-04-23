use crate::commands::index::index::Index;
use crate::storage::utils::{HEAD_DIR, INDEX_FILE, OBJ_DIR, REFS_DIR, VOX_DIR};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn init_command() -> Result<()> {
    fs::create_dir_all(&*VOX_DIR).context("Failed to create .vox directory")?;
    fs::create_dir_all(&*OBJ_DIR).context("Failed to create .vox/objects directory")?;
    fs::create_dir_all(&*REFS_DIR).context("Failed to create .vox/refs directory")?;
    fs::write(&*HEAD_DIR, "ref: refs/heads/main\n").context("Failed to write to .vox/HEAD file")?;

    let index = Index::new();
    index
        .write_to_file(Path::new(&*INDEX_FILE))
        .context("Failed to create index file")?;

    println!("Initialized vox directory");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::commands::init::init_command;
    use assert_cmd::Command;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_init_command() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let vox_dir = temp_dir.path().join(".vox");
        let obj_dir = vox_dir.join("objects");
        let refs_dir = vox_dir.join("refs");
        let head_file = vox_dir.join("HEAD");
        let index_file = vox_dir.join("index");

        std::env::set_current_dir(temp_dir.path()).unwrap();

        init_command().unwrap();

        assert!(vox_dir.exists());
        assert!(obj_dir.exists());
        assert!(refs_dir.exists());
        assert!(head_file.exists());
        assert!(index_file.exists());
    }
}
