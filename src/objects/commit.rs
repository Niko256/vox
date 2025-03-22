use super::objects::{Loadable, Storable, VoxObject};
use super::tree::Tree;
use crate::objects::delta::Delta;
use crate::utils::OBJ_DIR;
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
    pub tree: String,
    pub parent: Option<String>,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl VoxObject for Commit {
    fn object_type(&self) -> &str {
        "commit"
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = Vec::new();

        content.extend(format!("tree {}\n", self.tree).as_bytes());

        if let Some(parent) = &self.parent {
            content.extend(format!("parent {}\n", parent).as_bytes());
        }

        let timestamp = self.timestamp.timestamp().to_string();
        content.extend(format!("author {} {}\n", self.author, timestamp).as_bytes());
        content.extend(b"\n");

        content.extend(self.message.as_bytes());
        content.extend(b"\n");

        Ok(content)
    }

    fn hash(&self) -> Result<String> {
        let content = self.serialize()?;
        let mut hasher = Sha1::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn object_path(&self) -> Result<String> {
        let hash = self.hash()?;
        Ok(format!(
            "{}/{}/{}",
            OBJ_DIR.display(),
            &hash[..2],
            &hash[2..]
        ))
    }
}

impl Storable for Commit {
    fn save(&self, objects_dir: &PathBuf) -> Result<String> {
        let hash = self.hash()?;
        let content = self.serialize()?;

        let header = format!("commit {}\0", content.len());
        let full_content = [header.as_bytes(), &content].concat();

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_content)?;
        let compressed_data = encoder.finish()?;

        let dir_path = objects_dir.join(&hash[..2]);
        fs::create_dir_all(&dir_path)?;
        let object_path = dir_path.join(&hash[2..]);
        fs::write(&object_path, compressed_data)?;

        Ok(hash)
    }
}

impl Loadable for Commit {
    fn load(hash: &str, objects_dir: &PathBuf) -> Result<Self> {
        let dir_path = objects_dir.join(&hash[..2]);
        let object_path = dir_path.join(&hash[2..]);

        let compressed_data = fs::read(&object_path)?;
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let null_pos = decompressed_data
            .iter()
            .position(|&b| b == 0)
            .context("Invalid format: no null byte found")?;

        let header = std::str::from_utf8(&decompressed_data[..null_pos])?;
        if !header.starts_with("commit ") {
            return Err(anyhow::anyhow!("Not a commit object"));
        }

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
        let timestamp = Utc::now();
        Self {
            tree: tree_hash,
            parent: parent_hash,
            author,
            timestamp,
            message,
        }
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
                .ok_or_else(|| anyhow::anyhow!("Invalid commit format"))?;
            match key {
                "tree" => tree = Some(value.to_string()),
                "parent" => parent = Some(value.to_string()),
                "author" => {
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

        Ok(Self {
            tree: tree.context("Missing tree hash")?,
            parent,
            author: author.context("Missing author")?,
            timestamp: timestamp.context("Missing timestamp")?,
            message: message.join("\n"),
        })
    }
}

pub fn compare_commits(from_hash: &str, to_hash: &str, objects_dir: &PathBuf) -> Result<Delta> {
    let from_commit = Commit::load(from_hash, objects_dir)?;
    let to_commit = Commit::load(to_hash, objects_dir)?;

    let from_tree = Tree::load(&from_commit.tree, objects_dir)?;
    let to_tree = Tree::load(&to_commit.tree, objects_dir)?;

    let mut delta = Tree::compare_trees(&from_tree, &to_tree, objects_dir)?;

    delta.set_from(Some(from_hash.to_string()));
    delta.set_to(Some(to_hash.to_string()));

    Ok(delta)
}
