use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};
use anyhow::{Result, Context};
use crate::utils::OBJ_DIR;
use std::fs::{self, File};
use std::path::Path;
use std::io::{self, Write};
use super::blob::create_blob;


#[derive(Debug)]
pub struct TreeEntry {
    pub permissions: String,
    pub object_type: String,
    pub object_hash: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

pub fn create_tree(path: &Path) -> Result<Tree> {
    let mut tree = Tree { entries: Vec::new() };

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry_path.file_name().unwrap().to_str().unwrap().to_string();

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
            let subtree_hash = store_tree(&subtree)?;
            tree.entries.push(TreeEntry {
                object_type: "tree".to_string(),
                permissions: "40000".to_string(),
                object_hash: subtree_hash,
                name,
            });
        }
    }

    Ok(tree)
}


pub fn store_tree(tree: &Tree) -> Result<String> {
    let mut content = Vec::new();

    for entry in &tree.entries {
        let entry_line = format!("{} {} {}\0", entry.permissions, entry.object_type, entry.object_hash, entry.name);
        content.extend_from_slice(entry_line.as_bytes());
    }

    // Hashing the content using SHA-1
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let object_hash = format!("{:x}", hasher.finalize());

    // Compressing the content along with a header using Zlib
    let header = format!("tree {}\0", content.len());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(header.as_bytes()).context("Failed to write header to encoder")?;
    encoder.write_all(&content).context("Failed to write content to encoder")?;

    let compressed_data = encoder.finish().context("Failed to finish compression")?;

    // Writing the compressed data to the object file
    let object_path = format!("{}/{}/{}", *OBJ_DIR, &object_hash[0..2], &object_hash[2..]);
    std::fs::create_dir_all(format!("{}/{}", *OBJ_DIR, &object_hash[0..2])).context("Failed to create object directory")?;

    let mut object_file = File::create(&object_path).context("Failed to create object file")?;
    object_file.write_all(&compressed_data).context("Failed to write compressed data to file")?;

    Ok(object_hash)
}



pub fn read_tree(object_hash: &str) -> Result<Tree> {
    let object_path = format!("{}/{}/{}", *OBJ_DIR, &object_hash[0..2], &object_hash[2..]);
    let file = File::open(&object_path).with_context(|| format!("Failed to open object file: {}", object_hash))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut decoder_data = Vec::new();
    decoder.read_to_end(&mut decoder_data).context("Failed to read object data")?;

    // Split the header and the data
    let header_end = decoder_data.iter().position(|&b| b == b'\0').context("Failed to find header end")?;
    let data = &decoder_data[header_end + 1..];

    let mut tree = Tree { entries: Vec::new() };
    let mut data_iter = data.split(|&b| b == b'\0');

    while let Some(entry_data) = data_iter.next() {
        let entry_parts: Vec<&[u8]> = entry_data.splitn(4, |&b| b == b' ').collect();
        if entry_parts.len() == 4 {
            let permissions = String::from_utf8_lossy(entry_parts[0]).to_string();
            let object_type = String::from_utf8_lossy(entry_parts[1]).to_string();
            let object_hash = String::from_utf8_lossy(entry_parts[2]).to_string();
            let name = String::from_utf8_lossy(entry_parts[3]).to_string();
            tree.entries.push(TreeEntry {
                permissions,
                object_type,
                object_hash,
                name,
            });
        }
    }

    Ok(tree)
}
