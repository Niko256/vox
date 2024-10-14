use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_vcs_add() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let mut cmd = Command::cargo_bin("vcs").expect("Failed to find binary");
    cmd.arg("add").arg(file_path.to_str().unwrap()).current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added file to index"));

    let index_path = temp_dir.path().join(".vcs/index");
    assert!(index_path.exists());
    let index_content = fs::read_to_string(index_path).expect("Failed to read index file");
    assert!(index_content.contains("test_file.txt"));
}

#[test]
fn test_vcs_add_all() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path1 = temp_dir.path().join("test_file1.txt");
    let file_path2 = temp_dir.path().join("test_file2.txt");
    fs::write(&file_path1, "test content 1").expect("Failed to write file");
    fs::write(&file_path2, "test content 2").expect("Failed to write file");

    let mut cmd = Command::cargo_bin("vcs").expect("Failed to find binary");
    cmd.arg("add").arg("-A").current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added all files to index"));

    let index_path = temp_dir.path().join(".vcs/index");
    assert!(index_path.exists());
    let index_content = fs::read_to_string(index_path).expect("Failed to read index file");
    assert!(index_content.contains("test_file1.txt"));
    assert!(index_content.contains("test_file2.txt"));
}


#[test]
fn test_vcs_add_integration() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let mut init_cmd = Command::cargo_bin("vcs").expect("Failed to find binary");
    init_cmd.arg("init").current_dir(&temp_dir);
    init_cmd.assert().success();

    let mut add_cmd = Command::cargo_bin("vcs").expect("Failed to find binary");
    add_cmd.arg("add").arg(file_path.to_str().unwrap()).current_dir(&temp_dir);

    add_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added file to index"));

    let index_path = temp_dir.path().join(".vcs/index");
    assert!(index_path.exists());
    let index_content = fs::read_to_string(index_path).expect("Failed to read index file");
    assert!(index_content.contains("test_file.txt"));
}

#[test]
fn test_vcs_add_all_integration() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path1 = temp_dir.path().join("test_file1.txt");
    let file_path2 = temp_dir.path().join("test_file2.txt");
    fs::write(&file_path1, "test content 1").expect("Failed to write file");
    fs::write(&file_path2, "test content 2").expect("Failed to write file");

    let mut init_cmd = Command::cargo_bin("vcs").expect("Failed to find binary");
    init_cmd.arg("init").current_dir(&temp_dir);
    init_cmd.assert().success();

    let mut add_cmd = Command::cargo_bin("vcs").expect("Failed to find binary");
    add_cmd.arg("add").arg("-A").current_dir(&temp_dir);

    add_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added all files to index"));

    let index_path = temp_dir.path().join(".vcs/index");
    assert!(index_path.exists());
    let index_content = fs::read_to_string(index_path).expect("Failed to read index file");
    assert!(index_content.contains("test_file1.txt"));
    assert!(index_content.contains("test_file2.txt"));
}
