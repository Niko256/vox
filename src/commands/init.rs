use crate::utils::{HEAD_DIR, OBJ_DIR, REFS_DIR, VCS_DIR};
use anyhow::{Context, Result};
use std::fs;

pub fn init_command() -> Result<()> {
    fs::create_dir_all(&*VCS_DIR).context("Failed to create .vcs directory")?;
    fs::create_dir_all(&*OBJ_DIR).context("Failed to create .vcs/objects directory")?;
    fs::create_dir_all(&*REFS_DIR).context("Failed to create .vcs/refs directory")?;
    fs::write(&*HEAD_DIR, "ref: refs/heads/main\n").context("Failed to write to .vcs/HEAD file")?;

    println!("Initialized vcs directory");
    Ok(())
}

#[cfg(test)]
mod tests {

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

        // Run the init command
        Command::cargo_bin("vcs")
            .unwrap()
            .arg("init")
            .current_dir(&temp_dir)
            .assert()
            .success()
            .stdout("Initialized vcs directory\n");

        // Check that the directories and file were created
        assert!(vcs_dir.exists());
        assert!(obj_dir.exists());
        assert!(refs_dir.exists());
        assert!(head_file.exists());

        // Check the content of the HEAD file
        let head_content = fs::read_to_string(&head_file).expect("Failed to read HEAD file");
        assert_eq!(head_content, "ref: refs/heads/main\n");
    }
}
