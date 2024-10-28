use std::fs;
use crate::commands::index::update_index;
use anyhow::Result;


pub fn add_command(all: bool, file_path: Option<String>) -> Result<()> {
    if all {
        add_all_files()?;
        println!("Added all files to index");
    } else if let Some(path) = file_path {
        add_file(&path)?;
        println!("Added file to index");
    } else {
        return Err(anyhow::anyhow!("No file specified for addition"));
    }
    Ok(())
}

pub fn add_file(file_path: &str) -> Result<()> {
    let object_hash = crate::objects::blob::create_blob(file_path)?;
    update_index(file_path, &object_hash)?;
    Ok(())
}

pub fn add_all_files() -> Result<()> {
    let entries = fs::read_dir(".")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            add_file(path.to_str().unwrap())?;
        }
    }
    Ok(())
}
