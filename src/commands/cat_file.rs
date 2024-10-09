use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use flate2::read::ZlibDecoder;
use crate::utils::{OBJ_DIR};

pub fn cat_file_command(pretty_print: bool, object_hash: String, show_type: bool, show_size: bool) -> Result<()> {

    let object_path = format!("{}/{}/{}", *OBJ_DIR, &object_hash[0..2], &object_hash[2..]);
    let file = File::open(&object_path).with_context(|| format!("Failed to open object file: {}", object_hash))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut decoder_data = Vec::new();
    decoder.read_to_end(&mut decoder_data).context("Failed to read object data")?;

    // Split the header and the data
    let header_end = decoder_data.iter().position(|&b| b == b'\0').context("Failed to find header end")?;
    let header = String::from_utf8_lossy(&decoder_data[..header_end]);
    let data = &decoder_data[header_end + 1..];

    match (show_type, show_size, pretty_print) {
        (true, false, false) => {
            let object_type = header.split(' ').next().unwrap_or("unknown");
            println!("{}", object_type);
        },
        (false, true, false) => {
            println!("{}", data.len());
        },
        (false, false, true) | (false, false, false) => {
            print!("{}", String::from_utf8_lossy(data));
        },
        _ => {
            let object_type = header.split(' ').next().unwrap_or("unknown");
            println!("{}", object_type);
            println!("{}", data.len());
            print!("{}", String::from_utf8_lossy(data));
        }
    }

    Ok(())
}
