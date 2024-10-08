use std::fs;
use std::io::{self, Write};
use std::path::Path;
use tempfile::TempDir;
use assert_cmd::Command;
use flate2::write::ZlibEncoder;
use flate2::Compression;

#[test]
fn test_cat_file_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let vcs_dir = temp_dir.path().join(".vcs");
    let obj_dir = vcs_dir.join("objects");

    // Initialize the vcs directory
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("init")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("Initialized vcs directory\n");

    // Create a test object
    let object_hash = "1a2b3c4d5e6f7g8h9i0j";
    let object_path = obj_dir.join(&object_hash[0..2]).join(&object_hash[2..]);
    fs::create_dir_all(object_path.parent().unwrap()).expect("Failed to create object directory");

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let header = "blob 13\0";
    let data = "Hello, World!";
    encoder.write_all(header.as_bytes()).expect("Failed to write header");
    encoder.write_all(data.as_bytes()).expect("Failed to write data");
    let compressed_data = encoder.finish().expect("Failed to finish compression");

    fs::write(&object_path, compressed_data).expect("Failed to write object file");

    // Test cat-file with pretty print
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("cat-file")
        .arg("-p")
        .arg(object_hash)
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("Hello, World!");

    // Test cat-file with show type
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("cat-file")
        .arg("-t")
        .arg(object_hash)
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("blob\n");

    // Test cat-file with show size
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("cat-file")
        .arg("-s")
        .arg(object_hash)
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("13\n");

    // Test cat-file with show type and size
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("cat-file")
        .arg("-t")
        .arg("-s")
        .arg(object_hash)
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("blob\n13\nHello, World!");
}

#[test]
fn test_cat_file_command_invalid_hash() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize the vcs directory
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("init")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout("Initialized vcs directory\n");

    // Test cat-file with invalid hash
    Command::cargo_bin("vcs")
        .unwrap()
        .arg("cat-file")
        .arg("-p")
        .arg("invalid_hash")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicates::str::contains("Failed to open object file: invalid_hash"));
}
