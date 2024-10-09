use anyhow::{Context, Result};
use std::fs;
use crate::utils::{VCS_DIR, OBJ_DIR, REFS_DIR, HEAD_DIR};

pub fn init_command() -> Result<()> {
    fs::create_dir_all(&*VCS_DIR).context("Failed to create .vcs directory")?;
    fs::create_dir_all(&*OBJ_DIR).context("Failed to create .vcs/objects directory")?;
    fs::create_dir_all(&*REFS_DIR).context("Failed to create .vcs/refs directory")?;
    fs::write(&*HEAD_DIR, "ref: refs/heads/main\n").context("Failed to write to .vcs/HEAD file")?;

    println!("Initialized vcs directory");
    Ok(())
}
