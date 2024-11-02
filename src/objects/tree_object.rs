use super::blob::create_blob;
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::path::Path;

// Represents a single entry in a tree (similar to a directory entry)
#[derive(Debug)]
pub struct TreeEntry {
    pub permissions: String, // File permissions (e.g., "100644" for regular files)
    pub object_type: String, // Type of object ("blob" for files, "tree" for directories)
    pub object_hash: String, // SHA-1 hash of the object
    pub name: String,        // Name of the entry
}

// Represents a tree structure (similar to a directory)
#[derive(Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>, // Collection of entries in the tree
}

// Creates a tree structure from a given directory path
pub fn create_tree(path: &Path) -> Result<Tree> {
    let mut tree = Tree {
        entries: Vec::new(),
    };

    // Read all entries in the directory
    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Handle files and directories differently
        if entry_path.is_file() {
            // Create a blob for files
            let object_hash = create_blob(&entry_path.to_str().unwrap())?;
            tree.entries.push(TreeEntry {
                object_type: "blob".to_string(),
                permissions: "100644".to_string(), // Standard file permissions
                object_hash,
                name,
            });
        } else if entry_path.is_dir() {
            // Recursively create trees for directories
            let subtree = create_tree(&entry_path)?;
            let subtree_hash = store_tree(&subtree)?;
            tree.entries.push(TreeEntry {
                object_type: "tree".to_string(),
                permissions: "40000".to_string(), // Directory permissions
                object_hash: subtree_hash,
                name,
            });
        }
    }

    Ok(tree)
}

// Stores a tree object and returns its hash
pub fn store_tree(tree: &Tree) -> Result<String> {
    let mut content = Vec::new();

    // Format and store each entry
    for entry in &tree.entries {
        let mode_and_name = format!("{} {}\0", entry.permissions, entry.name);
        content.extend_from_slice(mode_and_name.as_bytes());

        let hash_bytes = hex::decode(&entry.object_hash)?;
        content.extend_from_slice(&hash_bytes);
    }

    // Create the full content with header
    let header = format!("tree {}", content.len());
    let full_content = [header.as_bytes(), b"\0", &content].concat();

    // Calculate SHA-1 hash
    let mut hasher = Sha1::new();
    hasher.update(&full_content);
    let hash = format!("{:x}", hasher.finalize());

    // Compress the content using zlib
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&full_content)?;
    let compressed = encoder.finish()?;

    // Store the compressed content
    let object_path = format!("{}/{}/{}", *OBJ_DIR, &hash[0..2], &hash[2..]);
    std::fs::create_dir_all(format!("{}/{}", *OBJ_DIR, &hash[0..2]))?;
    std::fs::write(object_path, compressed)?;

    Ok(hash)
}

// Reads a tree object from its hash
pub fn read_tree(hash: &str) -> Result<Tree> {
    // Read and decompress the tree object
    let object_path = format!("{}/{}/{}", *OBJ_DIR, &hash[0..2], &hash[2..]);
    let compressed = std::fs::read(object_path)?;

    let mut decoder = ZlibDecoder::new(Vec::new());
    decoder.write_all(&compressed)?;
    let data = decoder.finish()?;

    // Find the header separator
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .context("Invalid format: no null byte found")?;

    // Parse the content
    let content = &data[null_pos + 1..];
    let mut entries = Vec::new();
    let mut pos = 0;

    // Parse each entry
    while pos < content.len() {
        // Find the end of metadata
        let null_pos = content[pos..]
            .iter()
            .position(|&b| b == 0)
            .context("Invalid format: no null byte found in entry")?;

        // Parse metadata (permissions and name)
        let entry_meta = std::str::from_utf8(&content[pos..pos + null_pos])?;
        let (permissions, name) = entry_meta
            .split_once(' ')
            .context("Invalid format: no space in entry metadata")?;

        pos += null_pos + 1;

        // Read the object hash
        let hash_bytes = &content[pos..pos + 20];
        let object_hash = hex::encode(hash_bytes);
        pos += 20;

        // Create and store the entry
        entries.push(TreeEntry {
            permissions: permissions.to_string(),
            object_type: if permissions.starts_with("40") {
                "tree".to_string()
            } else {
                "blob".to_string()
            },
            object_hash,
            name: name.to_string(),
        });
    }

    Ok(Tree { entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::prelude::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_store_tree() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("file_1.txt"), "content1")?;
        fs::write(dir_path.join("file_2.txt"), "content2")?;

        let subdir_path = dir_path.join("subdir");
        fs::create_dir(&subdir_path)?;
        fs::write(subdir_path.join("file3.txt"), "content3")?;

        let tree = create_tree(dir_path)?;

        assert_eq!(tree.entries.len(), 3, "Tree should have 3 entries");

        let tree_hash = store_tree(&tree)?;

        let read_tree = read_tree(&tree_hash)?;

        assert_eq!(
            tree.entries.len(),
            read_tree.entries.len(),
            "Number of entries should match"
        );

        for (original, read) in tree.entries.iter().zip(read_tree.entries.iter()) {
            assert_eq!(
                original.permissions, read.permissions,
                "Permissions should match"
            );
            assert_eq!(
                original.object_type, read.object_type,
                "Object type should match"
            );
            assert_eq!(
                original.object_hash, read.object_hash,
                "Object hash should match"
            );
            assert_eq!(original.name, read.name, "Name should match");
        }

        Ok(())
    }

    #[test]
    fn test_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let tree = create_tree(&temp_dir.path())?;
        assert_eq!(
            tree.entries.len(),
            0,
            "Empty directory should create empty file"
        );
        Ok(())
    }

    #[test]
    fn test_nested_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();

        fs::create_dir(dir_path.join("dir_1"))?;
        fs::create_dir(dir_path.join("dir_1/dir_2"))?;
        fs::write(dir_path.join("dir_1/dir_2/tmp_file.txt"), "content")?;

        let tree = create_tree(dir_path)?;
        assert_eq!(tree.entries.len(), 1, "Root should have one entry");

        let dir_1_entry = &tree.entries[0];
        assert_eq!(dir_1_entry.permissions, "40000");
        assert_eq!(dir_1_entry.object_type, "tree");

        let dir_1_tree = read_tree(&dir_1_entry.object_hash)?;
        assert_eq!(dir_1_tree.entries.len(), 1);

        Ok(())
    }

    #[test]
    fn test_file_permissions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();

        // Creating txt file
        fs::write(dir_path.join("regular.txt"), "content")?;

        let tree = create_tree(dir_path)?;

        let regular_file = tree
            .entries
            .iter()
            .find(|e| e.name == "regular.txt")
            .expect("Regular file not found");
        assert_eq!(regular_file.permissions, "100644");

        Ok(())
    }
}
