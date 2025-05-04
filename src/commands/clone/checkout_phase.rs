use crate::commands::branch::checkout::checkout_command;
use crate::storage::utils::{HEAD_DIR, REFS_DIR};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct CloneCheckout;

impl CloneCheckout {
    pub fn execute(commit_hash: &str, workdir: &Path) -> Result<()> {
        let temp_branch = "clone_temp";
        Self::create_temp_branch(commit_hash, temp_branch)?;

        checkout_command(temp_branch, true)?;

        Self::rename_branch(temp_branch, "master")?;

        Ok(())
    }

    fn create_temp_branch(commit_hash: &str, name: &str) -> Result<()> {
        let ref_path = REFS_DIR.join("heads").join(name);
        if let Some(parent) = ref_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(ref_path, format!("{}\n", commit_hash))?;
        Ok(())
    }

    fn rename_branch(from: &str, to: &str) -> Result<()> {
        let from_path = REFS_DIR.join("heads").join(from);
        let to_path = REFS_DIR.join("heads").join(to);
        fs::rename(from_path, to_path)?;

        let head_content = fs::read_to_string(&*HEAD_DIR)?;
        if head_content.contains(from) {
            fs::write(&*HEAD_DIR, format!("ref: refs/heads/{}\n", to))?;
        }
        Ok(())
    }
}
