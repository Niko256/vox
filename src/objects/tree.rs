use super::blob::create_blob;
use super::delta::Delta;
use super::object::VoxObject;
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use hex;
use sha1::{Digest, Sha1};
use std::fs::{self, Permissions};
use std::io::Write;
use std::path::{Path, PathBuf}; // Импортируйте crate hex

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

impl Tree {
    fn new(permissions: String, object_type: String, object_hash: String, name: String) -> Self {
        Tree {
            entries: (permissions, object_type, object_hash, name),
        }
    }

    pub fn load(tree_hash: &str, object_dir: &PathBuf) -> Result<Self> {
        read_tree(tree_hash, object_dir)?;
    }

    pub fn compare_trees(from: &Tree, to: &Tree, objects_dir: &PathBuf) -> Result<Delta> {}
}

impl VoxObject for Tree {
    fn object_type(&self) -> &str {
        "tree"
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = Vec::new();

        // Format and store each entry
        for entry in &self.entries {
            let mode_and_name = format!("{} {}\0", entry.permissions, entry.name);
            content.extend_from_slice(mode_and_name.as_bytes());

            let hash_bytes = hex::decode(&entry.object_hash).expect("Decoding failed");

            content.extend_from_slice(&hash_bytes);
        }
        Ok(content)
    }

    fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(&self.serialize());
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn object_path(&self) -> Result<String> {
        let hash = self.hash();
        Ok(format!("{}/{}/{}", *OBJ_DIR, &hash[0..2], &hash[2..]))
    }
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

        if name.starts_with('.') || name == "target" {
            continue;
        }

        // Handle files and directories differently
        if entry_path.is_file() {
            let object_hash = create_blob(&entry_path.to_str().unwrap())?;
            tree.entries.push(TreeEntry {
                object_type: "blob".to_string(),
                permissions: "100644".to_string(),
                object_hash,
                name,
            });
        } else if entry_path.is_dir() {
            let subtree = create_tree(&entry_path)?;
            if !subtree.entries.is_empty() {
                let hash_str = subtree.hash(); // hash new functions;
                let subtree_hash = hash_str;
                tree.entries.push(TreeEntry {
                    object_type: "tree".to_string(),
                    permissions: "40000".to_string(),
                    object_hash: subtree_hash,
                    name,
                });
            }
        }
    }

    tree.entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tree)
}

// Stores a tree object and returns its hash
pub fn store_tree(tree: &Tree) -> Result<String> {
    let hash_str = tree.hash(); // new functions
    let object_path = tree.object_path(); // new functions

    let mut content = Vec::new();

    // Format and store each entry
    for entry in &tree.entries {
        let mode_and_name = format!("{} {}\0", entry.permissions, entry.name);
        content.extend_from_slice(mode_and_name.as_bytes());

        let hash_bytes = hex::decode(&entry.object_hash).expect("Decoding failed");
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

    if !std::path::Path::new(&object_path).exists() {
        println!("Creating new tree object: {}", hash);
        std::fs::create_dir_all(format!("{}/{}", *OBJ_DIR, &hash[0..2]))?;
        std::fs::write(object_path, compressed)?;
    }
    Ok(hash)
}

// Reads a tree object from its hash
pub fn read_tree(hash: &str, objects_dir: &PathBuf) -> Result<Tree> {
    // Read and decompress the tree object
    let object_path = objects_dir.join(&hash[0..2]).join(&hash[2..]);

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
