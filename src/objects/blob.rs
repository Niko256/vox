use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};

pub fn create_blob(file_path: &str) -> Result<String> {
    // reading the content of the file
    let mut file = File::open(&file_path).context("Failed to open file")?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .context("Failed to read file content")?;

    // hashing the content using SHA-1
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let object_hash = format!("{:x}", hasher.finalize());

    // compressing the content along with a header using Zlib
    let header = format!("blob {}\0", content.len());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(header.as_bytes())
        .context("Failed to write header to encoder")?;
    encoder
        .write_all(&content)
        .context("Failed to write content to encoder")?;

    let compressed_data = encoder.finish().context("Failed to finish compression")?;

    // writing the compressed data to the object file
    let object_path = format!("{}/{}/{}", *OBJ_DIR, &object_hash[0..2], &object_hash[2..]);
    std::fs::create_dir_all(format!("{}/{}", *OBJ_DIR, &object_hash[0..2]))
        .context("Failed to create object directory")?;

    let mut object_file = File::create(&object_path).context("Failed to create object file")?;
    object_file
        .write_all(&compressed_data)
        .context("Failed to write compressed data to file")?;

    Ok(object_hash)
}
