use std::fs;
use tempfile::TempDir;
use assert_cmd::Command;

#[test]
fn test_write_tree() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let file1_path = temp_path.join("file1.txt");
    let file2_path = temp_path.join("file2.txt");
    fs::write(&file1_path, "Hi!")?;
    fs::write(&file2_path, "Today is 28.10.2024")?;

    Command::cargo_bin("vcs")?
        .arg("add")
        .arg(file1_path.to_str().unwrap())
        .arg(file2_path.to_str().unwrap())
        .current_dir(&temp_path)
        .assert()
        .success();

    let output = Command::cargo_bin("vcs")?
        .arg("write-tree")
        .current_dir(&temp_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tree_hash = String::from_utf8(output)?.trim().to_string();

    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    Command::new("git")
        .arg("add")
        .arg(file1_path.to_str().unwrap())
        .arg(file2_path.to_str().unwrap())
        .current_dir(&temp_path)
        .assert()
        .success();

    let git_output = Command::new("git")
        .arg("write-tree")
        .current_dir(&temp_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let git_tree_hash = String::from_utf8(git_output)?.trim().to_string();

    assert_eq!(tree_hash, git_tree_hash);

    Ok(())
}
