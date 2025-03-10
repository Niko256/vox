use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use std::{fs::File, io::Read};

const HASH_PREFIX_LEN: usize = 2;
const HASH_BYTES_LEN: usize = 20;

/// Represents a vox object type
#[derive(Debug)]
enum VcsObjectType {
    Blob,
    Tree,
    Unknown(String),
}

impl VcsObjectType {
    fn from_str(s: &str) -> Self {
        match s {
            "blob" => Self::Blob,
            "tree" => Self::Tree,
            unknown => Self::Unknown(unknown.to_string()),
        }
    }
}

/// Represents a tree entry in vox
struct TreeEntry<'a> {
    mode: &'a str,
    name: &'a str,
    hash: String,
}

/// Handles the cat-file command functionality
pub fn cat_file_command(
    pretty_print: bool,
    object_hash: String,
    show_type: bool,
    show_size: bool,
) -> Result<()> {
    let object_data = read_vox_object(&object_hash)?;
    let (object_type, content) = parse_object_header(&object_data)?;

    match (show_type, show_size, pretty_print) {
        (true, false, false) => display_type(&object_type),
        (false, true, false) => display_size(content),
        (false, false, _) => display_content(&object_type, content)?,
        _ => display_all(&object_type, content)?,
    }

    Ok(())
}

fn read_vox_object(hash: &str) -> Result<Vec<u8>> {
    let object_path = format!(
        "{}/{}/{}",
        *OBJ_DIR,
        &hash[..HASH_PREFIX_LEN],
        &hash[HASH_PREFIX_LEN..]
    );

    let file = File::open(&object_path)
        .with_context(|| format!("Failed to open object file: {}", hash))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut data = Vec::new();
    decoder
        .read_to_end(&mut data)
        .context("Failed to read object data")?;

    Ok(data)
}

fn parse_object_header(data: &[u8]) -> Result<(VcsObjectType, &[u8])> {
    let header_end = data
        .iter()
        .position(|&b| b == b'\0')
        .context("Failed to find header end")?;

    let header = String::from_utf8_lossy(&data[..header_end]);
    let object_type = header
        .split(' ')
        .next()
        .map(VcsObjectType::from_str)
        .unwrap_or(VcsObjectType::Unknown("unknown".to_string()));

    Ok((object_type, &data[header_end + 1..]))
}

fn display_type(object_type: &VcsObjectType) {
    match object_type {
        VcsObjectType::Blob => println!("blob"),
        VcsObjectType::Tree => println!("tree"),
        VcsObjectType::Unknown(t) => println!("{}", t),
    }
}

fn display_size(content: &[u8]) {
    println!("{}", content.len());
}

fn display_content(object_type: &VcsObjectType, content: &[u8]) -> Result<()> {
    match object_type {
        VcsObjectType::Blob => print!("{}", String::from_utf8_lossy(content)),
        VcsObjectType::Tree => display_tree_content(content)?,
        VcsObjectType::Unknown(t) => return Err(anyhow::anyhow!("Unknown object type: {}", t)),
    }
    Ok(())
}

fn display_tree_content(data: &[u8]) -> Result<()> {
    let mut pos = 0;
    while pos < data.len() {
        let entry = parse_tree_entry(&data[pos..]).context("Failed to parse tree entry")?;

        println!(
            "{} {} {}\t{}",
            entry.mode,
            if entry.mode.starts_with("40") {
                "tree"
            } else {
                "blob"
            },
            entry.hash,
            entry.name
        );

        pos += entry.name.len() + entry.mode.len() + HASH_BYTES_LEN + 2; // +2 for null byte and space
    }
    Ok(())
}

fn parse_tree_entry(data: &[u8]) -> Result<TreeEntry> {
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .context("Invalid format: no null byte found in entry")?;

    let entry_meta = std::str::from_utf8(&data[..null_pos])?;
    let (mode, name) = entry_meta
        .split_once(' ')
        .context("Invalid format: no space in entry metadata")?;

    let hash_start = null_pos + 1;
    let hash_end = hash_start + HASH_BYTES_LEN;
    let hash = hex::encode(&data[hash_start..hash_end]);

    Ok(TreeEntry { mode, name, hash })
}

fn display_all(object_type: &VcsObjectType, content: &[u8]) -> Result<()> {
    display_type(object_type);
    display_size(content);
    display_content(object_type, content)?;
    Ok(())
}
