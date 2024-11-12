use anyhow::{Context, Ok, Result};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const INDEX_SIGNATURE: &[u8; 4] = b"DIRC";
const INDEX_VERSION: u32 = 2;

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub mtime: u64,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub hash: [u8; 20],
    pub flags: u16,
    pub path: PathBuf,
}

#[derive(Debug, Default)]
pub struct Index {
    pub entries: HashMap<PathBuf, IndexEntry>,
}

impl IndexEntry {
    pub fn new(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let _now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        Ok(IndexEntry {
            mtime: metadata.mtime() as u64,
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            size: metadata.size() as u32,
            hash: [0; 20],
            flags: 0,
            path: path.to_path_buf(),
        })
    }
}

impl Index {
    pub fn new() -> Self {
        Index {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn remove_entry(&mut self, path: &Path) -> Option<IndexEntry> {
        self.entries.remove(path)
    }

    pub fn get_entry(&self, path: &Path) -> Option<&IndexEntry> {
        let normalized_path = if path.starts_with("./") {
            path.strip_prefix("./").unwrap_or(path)
        } else {
            path
        };
        self.entries.get(normalized_path)
    }

    pub fn get_entries(&self) -> &HashMap<PathBuf, IndexEntry> {
        &self.entries
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory at {:?}", parent))?;
        }

        let mut file = File::create(path)
            .with_context(|| format!("Failed to create index file at {:?}", path))?;

        file.write_all(INDEX_SIGNATURE)
            .context("Failed to write index signature")?;
        file.write_all(&INDEX_VERSION.to_be_bytes())
            .context("Failed to write index version")?;
        file.write_all(&(self.entries.len() as u32).to_be_bytes())
            .context("Failed to write entries count")?;

        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        for entry in entries {
            file.write_all(&entry.mtime.to_be_bytes())
                .context("Failed to write entry mtime")?;
            file.write_all(&entry.dev.to_be_bytes())
                .context("Failed to write entry dev")?;
            file.write_all(&entry.ino.to_be_bytes())
                .context("Failed to write entry ino")?;
            file.write_all(&entry.uid.to_be_bytes())
                .context("Failed to write entry uid")?;
            file.write_all(&entry.gid.to_be_bytes())
                .context("Failed to write entry gid")?;
            file.write_all(&entry.mode.to_be_bytes())
                .context("Failed to write entry mode")?;
            file.write_all(&entry.size.to_be_bytes())
                .context("Failed to write entry size")?;
            file.write_all(&entry.hash)
                .context("Failed to write entry hash")?;
            file.write_all(&entry.flags.to_be_bytes())
                .context("Failed to write entry flags")?;

            let path_str = entry
                .path
                .to_str()
                .context("Failed to convert path to string")?;
            file.write_all(path_str.as_bytes())
                .context("Failed to write entry path")?;
            file.write_all(&[0])
                .context("Failed to write path terminator")?;
        }

        Ok(())
    }

    pub fn read_from_file(&mut self, path: &Path) -> Result<()> {
        let mut file =
            File::open(path).with_context(|| format!("Failed to open index file at {:?}", path))?;
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature)
            .context("Failed to read index signature")?;

        if &signature != INDEX_SIGNATURE {
            return Err(anyhow::anyhow!("Invalid index file signature"));
        }

        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)
            .context("Failed to read index version")?;
        let version = u32::from_be_bytes(version_bytes);
        if version != INDEX_VERSION {
            return Err(anyhow::anyhow!("Unsupported index version"));
        }

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let count = u32::from_be_bytes(count_bytes);

        self.entries.clear();
        for _ in 0..count {
            let mut entry = IndexEntry {
                mtime: 0,
                dev: 0,
                ino: 0,
                mode: 0,
                uid: 0,
                gid: 0,
                size: 0,
                hash: [0; 20],
                flags: 0,
                path: PathBuf::new(),
            };

            let mut buffer_u64 = [0u8; 8];
            file.read_exact(&mut buffer_u64)?;
            entry.mtime = u64::from_be_bytes(buffer_u64);

            let mut buffer = [0u8; 4];
            file.read_exact(&mut buffer)?;
            entry.dev = u32::from_be_bytes(buffer);

            file.read_exact(&mut buffer)?;
            entry.ino = u32::from_be_bytes(buffer);

            file.read_exact(&mut buffer)?;
            entry.mode = u32::from_be_bytes(buffer);

            file.read_exact(&mut buffer)?;
            entry.uid = u32::from_be_bytes(buffer);

            file.read_exact(&mut buffer)?;
            entry.gid = u32::from_be_bytes(buffer);

            file.read_exact(&mut buffer)?;
            entry.size = u32::from_be_bytes(buffer);

            file.read_exact(&mut entry.hash)?;

            let mut flag_bytes = [0u8; 2];
            file.read_exact(&mut flag_bytes)?;
            entry.flags = u16::from_be_bytes(flag_bytes);

            let mut path_bytes = Vec::new();
            let mut byte = [0u8; 1];

            loop {
                file.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break;
                }
                path_bytes.push(byte[0]);
            }

            entry.path = PathBuf::from(String::from_utf8_lossy(&path_bytes).into_owned());
            self.entries.insert(entry.path.clone(), entry);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_new_index() {
        let index = Index::new();
        assert_eq!(index.entries.len(), 0);
    }

    #[test]
    fn test_add_and_get_entry() {
        let mut index = Index::new();
        let entry = IndexEntry {
            mtime: 12345,
            dev: 1,
            ino: 2,
            mode: 0o100644,
            uid: 1000,
            gid: 1000,
            size: 100,
            hash: [1; 20],
            flags: 0,
            path: PathBuf::from("test.txt"),
        };

        index.add_entry(entry.clone());

        let retrieved = index.get_entry(&PathBuf::from("test.txt"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().mtime, 12345);
    }

    #[test]
    fn test_remove_entry() {
        let mut index = Index::new();
        let entry = IndexEntry {
            mtime: 12345,
            dev: 1,
            ino: 2,
            mode: 0o100644,
            uid: 1000,
            gid: 1000,
            size: 100,
            hash: [1; 20],
            flags: 0,
            path: PathBuf::from("test.txt"),
        };

        index.add_entry(entry);

        let removed = index.remove_entry(&PathBuf::from("test.txt"));
        assert!(removed.is_some());
        assert!(index.get_entry(&PathBuf::from("test.txt")).is_none());
    }

    #[test]
    fn test_write_and_read_index() -> Result<()> {
        let dir = tempdir()?;
        let index_path = dir.path().join("index");

        let mut original_index = Index::new();
        let entry = IndexEntry {
            mtime: 12345,
            dev: 1,
            ino: 2,
            mode: 0o100644,
            uid: 1000,
            gid: 1000,
            size: 100,
            hash: [1; 20],
            flags: 0,
            path: PathBuf::from("test.txt"),
        };

        original_index.add_entry(entry);
        original_index.write_to_file(&index_path)?;

        let mut read_index = Index::new();
        read_index.read_from_file(&index_path)?;

        assert_eq!(original_index.entries.len(), read_index.entries.len());

        let original_entry = original_index
            .get_entry(&PathBuf::from("test.txt"))
            .unwrap();
        let read_entry = read_index.get_entry(&PathBuf::from("test.txt")).unwrap();

        assert_eq!(original_entry.mtime, read_entry.mtime);
        assert_eq!(original_entry.hash, read_entry.hash);
        assert_eq!(original_entry.path, read_entry.path);

        Ok(())
    }

    #[test]
    fn test_invalid_index_signature() -> Result<()> {
        let dir = tempdir()?;
        let index_path = dir.path().join("index");

        let mut file = File::create(&index_path)?;
        file.write_all(b"INVALID")?;

        let mut index = Index::new();
        let result = index.read_from_file(&index_path);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid index file signature"));

        Ok(())
    }

    #[test]
    fn test_index_version_check() -> Result<()> {
        let dir = tempdir()?;
        let index_path = dir.path().join("index");

        let mut file = File::create(&index_path)?;
        file.write_all(INDEX_SIGNATURE)?;
        file.write_all(&99u32.to_be_bytes())?;

        let mut index = Index::new();
        let result = index.read_from_file(&index_path);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported index version"));

        Ok(())
    }
}
