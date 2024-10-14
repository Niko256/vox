use clap::Parser;
use anyhow::{Result, Context};
use flate2::write::ZlibEncoder; 
use std::fs::File;
use std::io::{Read, Write};
use flate2::Compression;
use sha1::{Sha1, Digest};
use crate::utils::OBJ_DIR;


#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    pub file_path: String,
}


pub fn hash_object_command(args: HashObjectArgs) -> Result<()> {
    let object_hash = create_blob(&args.file_path)?;
    println!("{}", object_hash);
    Ok(())
}


pub fn create_blob(content: &[u8]) -> Result<String> {
    let header = format!("blob {}\0", content.len());
    let mut data = Vec::new();
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(content);

    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);

    let object_path = format!("{}/{}/{}", *OBJ_DIR, &hash_str[0..2], &hash_str[2..]);
    let mut encoder = ZlibEncoder::new(File::create(&object_path).context("Failed to create object file")?, Compression::default());
    encoder.write_all(&data).context("Failed to write object data")?;
    encoder.finish().context("Failed to finish writing object data")?;

    Ok(hash_str)
}
