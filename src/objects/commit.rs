use super::object::{Loadable, Storable, VoxObject};
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression; // Compression settings
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct Commit {
    pub tree: String,             // Hash of the tree object
    pub parent: Option<String>,   // Hash of the parent commit (None for initial commit)
    pub author: String,           // Author of the commit
    pub timestamp: DateTime<Utc>, // When the commit was created
    pub message: String,          // Commit message
}

impl VoxObject for Commit {
    fn object_type(&self) -> &str {
        "commit"
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = Vec::new();

        // Add tree ref
        content.extend(format!("tree {}\n", self.tree).as_bytes());

        // Add parent commit ref (if exists)
        if let Some(parent) = &self.parent {
            content.extend(format!("parent {}\n", parent).as_bytes());
        }

        // Add author and timestamp
        let timestamp = self.timestamp.timestamp().to_string();
        content.extend(format!("author {} {}\n", self.author, timestamp).as_bytes());
        content.extend(b"\n");

        // Add commit message
        content.extend(self.message.as_bytes());
        content.extend(b"\n");

        Ok(content)
    }

    fn hash(&self) -> Result<String> {
        let content = self.serialize();
        let mut hasher = Sha1::new();

        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn object_path(&self) -> Result<String> {
        let hash = self.hash();
        Ok(format!("{}/{}/{}", *OBJ_DIR, &hash[..2], &hash[2..]))
    }
}

impl Storable for Commit {
    // Saves commit object to disk in compressed format
    fn save(&self, objects_dir: &PathBuf) -> Result<String> {
        let hash = self.hash();
        let content = self.serialize();

        let header = format!("commit {}\0", content.len());
        let full_content = [header.as_bytes(), &content].concat();

        // Compress using zlib
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_content)?;
        let compressed_data = encoder.finish()?;

        // Save to filesystem using git-like structure (xx/yyyy...)
        let dir_path = objects_dir.join(&hash[..2]);
        fs::create_dir_all(&dir_path)?;
        let object_path = dir_path.join(&hash[2..]);
        fs::write(&object_path, compressed_data)?;

        Ok(hash)
    }
}

impl Loadable for Commit {
    // Loads and parses a commit object from disk by its hash
    fn load(hash: &str, objects_dir: &PathBuf) -> Result<Self> {
        // Construct path to object file
        let dir_path = objects_dir.join(&hash[..2]);
        let object_path = dir_path.join(&hash[2..]);

        // Read and decompress object data
        let compressed_data = fs::read(&object_path)?;
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        // Find the null byte separating header from content
        let null_pos = decompressed_data
            .iter()
            .position(|&b| b == 0)
            .context("Invalid format: no null byte found")?;

        // Verify object type
        let header = std::str::from_utf8(&decompressed_data[..null_pos])?;
        if !header.starts_with("commit ") {
            return Err(anyhow::anyhow!("Not a commit object"));
        }

        // Parse content
        let content = std::str::from_utf8(&decompressed_data[null_pos + 1..])?;
        Self::parse(content)
    }
}

impl Commit {
    pub fn new(
        tree_hash: String,
        parent_hash: Option<String>,
        author: String,
        message: String,
    ) -> Self {
        let timestamp = Utc::now(); // Get current timestamp
        Self {
            tree: tree_hash,
            parent: parent_hash,
            author,
            timestamp,
            message,
        }
    }

    // Parse commit content into a Commit struct
    fn parse(content: &str) -> Result<Self> {
        let mut lines = content.lines();
        let mut tree = None;
        let mut parent = None;
        let mut author = None;
        let mut timestamp = None;
        let mut message = Vec::new();
        let mut reading_message = false;

        // Parse header fields until empty line
        while let Some(line) = lines.next() {
            if reading_message {
                message.push(line.to_string());
                continue;
            }

            if line.is_empty() {
                reading_message = true;
                continue;
            }

            // Parse key-value pairs
            let (key, value) = line
                .split_once(' ')
                .ok_or_else(|| anyhow::anyhow!("Invalid commit format"))?;
            match key {
                "tree" => tree = Some(value.to_string()),
                "parent" => parent = Some(value.to_string()),
                "author" => {
                    // Parse author and timestamp
                    let parts: Vec<&str> = value.rsplitn(2, ' ').collect();
                    author = Some(parts[1].to_string());
                    timestamp = Some(
                        DateTime::from_timestamp(parts[0].parse::<i64>()?, 0)
                            .unwrap()
                            .with_timezone(&Utc),
                    );
                }
                _ => return Err(anyhow::anyhow!("Unknown commit field: {}", key)),
            }
        }

        // Construct and return Commit object
        Ok(Self {
            tree: tree.context("Missing tree hash")?,
            parent,
            author: author.context("Missing author")?,
            timestamp: timestamp.context("Missing timestamp")?,
            message: message.join("\n"),
        })
    }
}
