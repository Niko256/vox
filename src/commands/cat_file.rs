use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::Read;

pub fn cat_file_command(
    pretty_print: bool,
    object_hash: String,
    show_type: bool,
    show_size: bool,
) -> Result<()> {
    let object_path = format!("{}/{}/{}", *OBJ_DIR, &object_hash[0..2], &object_hash[2..]);
    let file = File::open(&object_path)
        .with_context(|| format!("Failed to open object file: {}", object_hash))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut decoder_data = Vec::new();
    decoder
        .read_to_end(&mut decoder_data)
        .context("Failed to read object data")?;

    // Split the header and the data
    let header_end = decoder_data
        .iter()
        .position(|&b| b == b'\0')
        .context("Failed to find header end")?;
    let header = String::from_utf8_lossy(&decoder_data[..header_end]);
    let data = &decoder_data[header_end + 1..];

    let object_type = header.split(' ').next().unwrap_or("unknown");

    match (show_type, show_size, pretty_print) {
        (true, false, false) => {
            println!("{}", object_type);
        }
        (false, true, false) => {
            println!("{}", data.len());
        }
        (false, false, true) | (false, false, false) => match object_type {
            "blob" => {
                print!("{}", String::from_utf8_lossy(data));
            }
            "tree" => {
                let mut pos = 0;
                while pos < data.len() {
                    let null_pos = data[pos..]
                        .iter()
                        .position(|&b| b == 0)
                        .context("Invalid format: no null byte found in entry")?;

                    let entry_meta = std::str::from_utf8(&data[pos..pos + null_pos])?;
                    let (mode, name) = entry_meta
                        .split_once(' ')
                        .context("Invalid format: no space in entry metadata")?;

                    pos += null_pos + 1;

                    let hash_bytes = &data[pos..pos + 20];
                    let hash = hex::encode(hash_bytes);
                    pos += 20;

                    println!(
                        "{} {} {}\t{}",
                        mode,
                        if mode.starts_with("40") {
                            "tree"
                        } else {
                            "blob"
                        },
                        hash,
                        name
                    );
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown object type: {}", object_type));
            }
        },
        _ => {
            println!("{}", object_type);
            println!("{}", data.len());
            print!("{}", String::from_utf8_lossy(data));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::blob::create_blob;
    use crate::objects::tree_object::{create_tree, store_tree};
    use anyhow::Result;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_cat_file_tree_output() -> Result<()> {
        use crate::commands::cat_file::cat_file_command;

        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("file1.txt"), "content1")?;
        fs::write(dir_path.join("file2.txt"), "content2")?;

        let tree = create_tree(dir_path)?;
        let hash = store_tree(&tree)?;

        cat_file_command(true, hash.clone(), false, false)?; // pretty-print
        cat_file_command(false, hash.clone(), true, false)?; // show type
        cat_file_command(false, hash.clone(), false, true)?; // show size

        Ok(())
    }

    #[test]
    fn test_cat_file_blob() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "test content\n";
        fs::write(&file_path, content)?;

        let hash = create_blob(file_path.to_str().unwrap())?;

        cat_file_command(false, hash.clone(), true, false)?;

        cat_file_command(false, hash.clone(), false, true)?;

        cat_file_command(true, hash.clone(), false, false)?;

        Ok(())
    }

    #[test]
    fn test_cat_file_tree() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("file1.txt"), "content1")?;

        fs::create_dir(dir_path.join("subdir"))?;
        fs::write(dir_path.join("subdir/file2.txt"), "content2")?;

        let tree = create_tree(dir_path)?;
        let hash = store_tree(&tree)?;

        cat_file_command(false, hash.clone(), true, false)?;

        cat_file_command(false, hash.clone(), false, true)?;

        cat_file_command(true, hash.clone(), false, false)?;

        Ok(())
    }

    #[test]
    fn test_cat_file_invalid_object() {
        let result = cat_file_command(true, "invalid_hash".to_string(), false, false);
        assert!(result.is_err());
    }
}
