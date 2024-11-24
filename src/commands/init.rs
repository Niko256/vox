use crate::commands::index::index::Index;
use crate::utils::{HEAD_DIR, INDEX_FILE, OBJ_DIR, REFS_DIR, VCS_DIR};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn init_command() -> Result<()> {
    fs::create_dir_all(&*VCS_DIR).context("Failed to create .vcs directory")?;
    fs::create_dir_all(&*OBJ_DIR).context("Failed to create .vcs/objects directory")?;
    fs::create_dir_all(&*REFS_DIR).context("Failed to create .vcs/refs directory")?;
    fs::write(&*HEAD_DIR, "ref: refs/heads/main\n").context("Failed to write to .vcs/HEAD file")?;

    let index = Index::new();
    index
        .write_to_file(Path::new(&*INDEX_FILE))
        .context("Failed to create index file")?;

    println!("Initialized vcs directory");
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
        let vcs_dir = temp_dir.path().join(".vcs");
        let obj_dir = vcs_dir.join("objects");
        let refs_dir = vcs_dir.join("refs");
        let head_file = vcs_dir.join("HEAD");
        let index_file = vcs_dir.join("index");

        std::env::set_current_dir(temp_dir.path()).unwrap();

        init_command().unwrap();

        assert!(vcs_dir.exists());
        assert!(obj_dir.exists());
        assert!(refs_dir.exists());
        assert!(head_file.exists());
        assert!(index_file.exists());
    }
}
