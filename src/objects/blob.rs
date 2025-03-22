use super::object::VoxObject;
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};

pub struct Blob {
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Blob { data }
    }

    pub fn from_file(file_path: &str) -> Result<Self> {
        // reading the content from the file
        let mut file = File::open(&file_path).context("Failed to open file")?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .context("Failed to read file content")?;

        Ok(Blob::new(content))
    }

    pub fn get_content(&self) -> &Vec<u8> {
        &self.data
    }
}

/// ----------------- BLOB IMPL -------------------

impl VoxObject for Blob {
    fn object_type(&self) -> &str {
        "blob"
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.get_content().clone())
    }

    fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(&self.serialize());
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn object_path(&self) -> Result<String> {
        let hash = self.hash();
        Ok(format!("{}/{}/{}", *OBJ_DIR, &hash[..2], &hash[2..]))
    }
}

pub fn create_blob(file_path: &str) -> Result<String> {
    let blob = Blob::from_file(file_path)?;
    let object_hash = blob.hash();

    // compressing the content along with a header using Zlib
    let header = format!("{} {}\0", blob.object_type(), blob.serialize().len());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(header.as_bytes())
        .context("Failed to write header to encoder")?;
    encoder
        .write_all(&blob.serialize())
        .context("Failed to write content to encoder")?;

    let compressed_data = encoder.finish().context("Failed to finish compression")?;

    // writing the compressed data to the object file
    let object_path = blob.object_path();
    std::fs::create_dir_all(format!("{}/{}", *OBJ_DIR, &object_hash[0..2]))
        .context("Failed to create object directory")?;

    let mut object_file = File::create(&object_path).context("Failed to create object file")?;
    object_file
        .write_all(&compressed_data)
        .context("Failed to write compressed data to file")?;

    Ok(object_hash)
}
