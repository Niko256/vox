use super::object::{Storable, VcsObject};
use crate::utils::OBJ_DIR;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;

pub struct Tag {
    id: String,
    target: String, // Hash of the object this tag points to
}

impl VcsObject for Tag {
    fn object_type(&self) -> &str {
        "tag"
    }

    fn serialize(&self) -> Vec<u8> {
        format!("id {}\ntarget {}\n", self.id, self.target)
            .as_bytes()
            .to_vec()
    }

    fn hash(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(&self.serialize());
        format!("{:x}", hasher.finalize())
    }

    fn object_path(&self) -> String {
        let hash = self.hash();
        format!("{}/{}/{}", *OBJ_DIR, &hash[..2], &hash[2..])
    }
}

impl Storable for Tag {
    fn save(&self, objects_dir: &std::path::PathBuf) -> anyhow::Result<String> {
        let hash = self.hash();
        let content = self.serialize();

        let header = format!("tag {}\0", content.len());
        let full_content = [header.as_bytes(), &content].concat();

        // Compression
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_content)?;
        let compressed_data = encoder.finish()?;

        // Saving to filesystem
        let dir_path = objects_dir.join(&hash[..2]);
        fs::create_dir_all(&dir_path)?;
        let object_path = dir_path.join(&hash[2..]);
        fs::write(&object_path, compressed_data)?;

        Ok(hash)
    }
}
