use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct Commit {
    tree: String,
    parent: Option<String>,
    author: String,
    timestamp: DateTime<Utc>,
    pub message: String,
}

impl Commit {
    pub fn new(
        tree_hash: String,
        parent_hash: Option<String>,
        author: String,
        message: String,
    ) -> Self {
        let timestamp = Utc::now();

        Self {
            tree: tree_hash,
            parent: parent_hash,
            author,
            timestamp,
            message,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut content = Vec::new();

        content.extend(format!("tree {}\n", self.tree).as_bytes());
        if let Some(parent) = &self.parent {
            content.extend(format!("parent {}\n", parent).as_bytes());
        }

        let timestamp_str = self.timestamp.format("%s %z").to_string();
        content.extend(format!("author {} {}\n", self.author, timestamp_str).as_bytes());
        content.extend(b"\n");
        content.extend(self.message.as_bytes());
        content.extend(b"\n");

        content
    }

    pub fn hash(&self) -> String {
        let content = self.serialize();
        let mut hasher = Sha1::new();
        hasher.update(&content);
        format!("{:x}", hasher.finalize())
    }

    pub fn save(&self, objects_dir: &PathBuf) -> Result<String> {
        let hash = self.hash();
        let content = self.serialize();

        // Create header and full content
        let header = format!("commit {}\0", content.len());
        let full_content = [header.as_bytes(), &content].concat();

        // Compress the content
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&full_content)
            .context("Failed to compress commit data")?;
        let compressed_data = encoder.finish().context("Failed to finish compression")?;

        // Save to file
        let dir_path = objects_dir.join(&hash[..2]);
        fs::create_dir_all(&dir_path).context("Failed to create object directory")?;

        let object_path = dir_path.join(&hash[2..]);
        fs::write(&object_path, compressed_data).context("Failed to write commit object")?;

        Ok(hash)
    }

    pub fn load(hash: &str, objects_dir: &PathBuf) -> Result<Self> {
        let dir_path = objects_dir.join(&hash[..2]);
        let object_path = dir_path.join(&hash[2..]);

        let compressed_data = fs::read(&object_path).context("Failed to read commit object")?;
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed_data = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .context("Failed to decompress commit data")?;

        let null_pos = decompressed_data
            .iter()
            .position(|&b| b == 0)
            .context("Invalid format: no null byte found")?;

        let header = std::str::from_utf8(&decompressed_data[..null_pos])
            .context("Invalid header encoding")?;

        if !header.starts_with("commit ") {
            return Err(anyhow::anyhow!("Not a commit object"));
        }

        let content = std::str::from_utf8(&decompressed_data[null_pos + 1..])
            .context("Invalid content encoding")?;
        Self::parse(content)
    }

    fn parse(content: &str) -> Result<Self> {
        let mut lines = content.lines();
        let mut tree = None;
        let mut parent = None;
        let mut author = None;
        let mut timestamp = None;
        let mut message = Vec::new();
        let mut reading_message = false;

        while let Some(line) = lines.next() {
            if reading_message {
                message.push(line.to_string());
                continue;
            }

            if line.is_empty() {
                reading_message = true;
                continue;
            }

            let (key, value) = line
                .split_once(' ')
                .context("Invalid commit format: line without space")?;

            match key {
                "tree" => tree = Some(value.to_string()),
                "parent" => parent = Some(value.to_string()),
                "author" => {
                    let parts: Vec<&str> = value.rsplitn(2, ' ').collect();
                    if parts.len() != 2 {
                        return Err(anyhow::anyhow!("Invalid author format"));
                    }
                    author = Some(parts[1].to_string());
                    timestamp = Some(
                        DateTime::parse_from_str(parts[0], "%s %z")
                            .context("Invalid timestamp format")?
                            .with_timezone(&Utc),
                    );
                }
                _ => return Err(anyhow::anyhow!("Unknown commit field: {}", key)),
            }
        }

        Ok(Self {
            tree: tree.context("Missing tree hash")?,
            parent,
            author: author.context("Missing author")?,
            timestamp: timestamp.context("Missing timestamp")?,
            message: message.join("\n"),
        })
    }
}
