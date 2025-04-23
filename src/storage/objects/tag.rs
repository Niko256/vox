use crate::storage::objects::{Storable, VoxObject};
use crate::storage::utils::{OBJ_DIR, OBJ_TYPE_TAG};
use anyhow::{anyhow, Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct Tag {
    id: String,
    target: String,
}

impl VoxObject for Tag {
    fn object_type(&self) -> &str {
        OBJ_TYPE_TAG
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(format!("id {}\ntarget {}\n", self.id, self.target)
            .as_bytes()
            .to_vec())
    }

    fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(&self.serialize()?);
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

impl Storable for Tag {
    fn save(&self, objects_dir: &Path) -> Result<String> {
        let hash = self.hash()?;
        let content = self.serialize()?;

        let header = format!("tag {}\0", content.len());
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

impl Tag {
    pub fn load(hash: &str, objects_dir: &Path) -> Result<Self> {
        let dir_path = objects_dir.join(&hash[..2]);
        let object_path = dir_path.join(&hash[2..]);

        let compressed_data = fs::read(&object_path)
            .with_context(|| format!("Failed to read tag object at {}", object_path.display()))?;

        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let data = String::from_utf8(decompressed_data)?;

        let mut id = None;
        let mut target = None;

        for line in data.lines() {
            if let Some(stripped) = line.strip_prefix("id ") {
                id = Some(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("target ") {
                target = Some(stripped.to_string());
            }
        }

        let id = id.ok_or_else(|| anyhow!("Missing 'id' in tag object"))?;
        let target = target.ok_or_else(|| anyhow!("Missing 'target' in tag object"))?;

        Ok(Tag { id, target })
    }
}
