use std::fs;
use tempfile::TempDir;
use assert_cmd::Command;

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

