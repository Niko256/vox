use super::objects::{Storable, VoxObject};
use crate::utils::{OBJ_DIR, OBJ_TYPE_BLOB};
use anyhow::{Context, Result};
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct Blob {
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(file_path: &str) -> Result<Self> {
        let data = std::fs::read(file_path)?;
        Ok(Blob { data })
    }

    pub fn blob_hash(file_path: &str) -> Result<String> {
        let blob = Blob::from_file(file_path)?;
        let object_hash = blob.hash()?;

        // Compressing the content along with a header using Zlib
        let header = format!("{} {}\0", blob.object_type(), blob.serialize()?.len());

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(header.as_bytes())
            .context("Failed to write header to encoder")?;
        encoder
            .write_all(&blob.serialize()?)
            .context("Failed to write content to encoder")?;

        let compressed_data = encoder.finish().context("Failed to finish compression")?;

        // Writing the compressed data to the object file
        let object_path = blob.object_path()?;
        std::fs::create_dir_all(format!("{}/{}", OBJ_DIR.display(), &object_hash[0..2]))
            .context("Failed to create object directory")?;

        let mut object_file = File::create(&object_path).context("Failed to create object file")?;
        object_file
            .write_all(&compressed_data)
            .context("Failed to write compressed data to file")?;

        Ok(object_hash)
    }

    pub fn from_file(file_path: &str) -> Result<Self> {
        // Reading the content from the file
        let mut file = File::open(file_path).context("Failed to open file")?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .context("Failed to read file content")?;

        Ok(Blob { data: content })
    }

    pub fn get_content(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn load(hash: &str, obj_dir: &Path) -> Result<Self> {
        let object_path = obj_dir.join(&hash[0..2]).join(&hash[2..]);
        let compressed = std::fs::read(object_path)?;

        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;

        let null_position = data
            .iter()
            .position(|&b| b == 0)
            .context("Invalid blob format: no null byte found")?;

        let content = &data[null_position + 1..];

        Ok(Blob {
            data: content.to_vec(),
        })
    }
}

impl VoxObject for Blob {
    fn object_type(&self) -> &str {
        OBJ_TYPE_BLOB
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.get_content().clone())
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

impl Storable for Blob {
    fn save(&self, objects_dir: &Path) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(&self.data);
        let hash = format!("{:x}", hasher.finalize());

        let header = format!("blob {}\0", self.data.len());
        let full_content = [header.as_bytes(), &self.data].concat();

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
