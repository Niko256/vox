use crate::objects::blob::create_blob;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    pub file_path: String,
}

pub fn hash_object_command(args: HashObjectArgs) -> Result<()> {
    let object_hash = create_blob(&args.file_path)?;
    println!("{}", object_hash);
    Ok(())
}

#[cfg(test)]
mod tests {

    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_hash_object() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "test content")?;

        let mut cmd = Command::cargo_bin("vcs")?;
        cmd.arg("hash-object").arg(file_path.to_str().unwrap());

        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(r"[a-f0-9]{40}").unwrap());

        Ok(())
    }

    #[test]
    fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("vcs")?;
        cmd.arg("help");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Usage: vcs <COMMAND>"));

        Ok(())
    }

    #[test]
    fn test_init_command() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let mut cmd = Command::cargo_bin("vcs")?;
        cmd.arg("init").current_dir(dir.path());

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Initialized vcs directory"));

        assert!(dir.path().join(".vcs").exists());
        assert!(dir.path().join(".vcs/objects").exists());
        assert!(dir.path().join(".vcs/refs").exists());
        assert!(dir.path().join(".vcs/HEAD").exists());

        Ok(())
    }

    #[test]
    fn test_integration() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let mut cmd = Command::cargo_bin("vcs")?;
        cmd.arg("init").current_dir(dir.path());
        cmd.assert().success();

        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "test content")?;

        let mut cmd = Command::cargo_bin("vcs")?;
        cmd.arg("hash-object").arg(file_path.to_str().unwrap());
        let output = cmd.output()?;
        let hash = String::from_utf8(output.stdout)?.trim().to_string();

        let mut cmd = Command::cargo_bin("vcs")?;
        cmd.arg("cat-file").arg("-p").arg(hash);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("test content"));

        Ok(())
    }
}
