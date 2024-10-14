use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write, Read};
use std::path::Path;
use crate::utils::INDEX_FILE;
use anyhow::Result;

pub fn update_index(file_path: &str, object_hash: &str) -> Result<()> {
    let mut index = load_index()?;
    index.insert(file_path.to_string(), object_hash.to_string());
    save_index(&index)?;
    Ok(())
}

pub fn load_index() -> Result<HashMap<String, String>> {
    let index_path = Path::new(&*INDEX_FILE);
    if !index_path.exists() {
        return Ok(HashMap::new());
    }

    let mut file = fs::File::open(index_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut index = HashMap::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() == 2 {
            index.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    Ok(index)
}

pub fn save_index(index: &HashMap<String, String>) -> Result<()> {
    let index_path = Path::new(&*INDEX_FILE);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(index_path)?;

    for (file_path, object_hash) in index {
        writeln!(file, "{} {}", file_path, object_hash)?;
    }

    Ok(())
}
