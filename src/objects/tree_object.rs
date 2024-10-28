use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use crate::utils::OBJ_DIR;


#[derive(Debug)]
struct TreeEntry {
    mode: String,
    object_type: String,
    object_hash: String,
    name: String,
}

impl TreeEntry {
    fn new(mode: String, object_type: String, object_hash: String, name: String) -> Self {
        TreeEntry {
            mode,
            object_type,
            object_hash,
            name,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        format!("{} {} {}\0", self.mode, self.object_type, self.object_hash)
            .as_bytes()
            .to_vec()
    }
}


pub fn create_tree_from_index(index: &[(String, String)]) -> Result<String> {
    let mut entries = Vec::new();

    for (file_path, object_hash) in index {
        let path = Path::new(file_path);
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        if path.is_file() {
            entries.push(TreeEntry::new("100644".to_string(), "blob".to_string(), object_hash.clone(), file_name));
        } else if path.is_dir() {
            let sub_index = index.iter()
                .filter(|(p, _)| p.starts_with(file_path) && p != file_path)
                .map(|(p, h)| (p.clone(), h.clone()))
                .collect::<Vec<_>>();
            let sub_tree_hash = create_tree_from_index(&sub_index)?;
            entries.push(TreeEntry::new("040000".to_string(), "tree".to_string(), sub_tree_hash, file_name));
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut content = Vec::new();
    for entry in entries {
        content.extend(entry.to_bytes());
    }

    let mut hasher = Sha1::new();
    hasher.update(&content);
    let object_hash = format!("{:x}", hasher.finalize());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&content)?;
    let compressed_data = encoder.finish()?;

    let object_path = format!("{}/{}/{}", *OBJ_DIR, &object_hash[0..2], &object_hash[2..]);
    fs::create_dir_all(format!("{}/{}", *OBJ_DIR, &object_hash[0..2]))?;
    let mut object_file = File::create(&object_path)?;
    object_file.write_all(&compressed_data)?;

    Ok(object_hash)
}
