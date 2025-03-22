use super::blob::Blob;
use super::delta::{Delta, DeltaType, FileDelta};
use super::objects::{Storable, VoxObject};
use crate::commands::diff::diff::text_diff;
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TreeEntry {
    pub permissions: String,
    pub object_type: String,
    pub object_hash: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn new(entries: Vec<TreeEntry>) -> Self {
        Self { entries }
    }

    pub fn load(tree_hash: &str, object_dir: &PathBuf) -> Result<Self> {
        read_tree(tree_hash, object_dir)
    }

    pub fn compare_trees(from: &Tree, to: &Tree, objects_dir: &PathBuf) -> Result<Delta> {
        let mut delta = Delta::default();

        let mut all_paths = HashSet::new();
        for entry in &from.entries {
            all_paths.insert(PathBuf::from(&entry.name));
        }
        for entry in &to.entries {
            all_paths.insert(PathBuf::from(&entry.name));
        }

        for path in all_paths {
            let from_entry = from
                .entries
                .iter()
                .find(|e| e.name == path.to_str().unwrap());
            let to_entry = to.entries.iter().find(|e| e.name == path.to_str().unwrap());

            match (from_entry, to_entry) {
                (None, Some(to)) => {
                    let blob = Blob::load(&to.object_hash, &objects_dir)?;
                    delta.add_file(
                        path.clone(),
                        FileDelta {
                            delta_type: DeltaType::Added,
                            old_path: None,
                            new_path: Some(path.clone()),
                            old_hash: None,
                            new_hash: Some(to.object_hash.clone()),
                            diff: Some(String::from_utf8_lossy(&blob.data).into_owned()),
                            added_lines: blob.data.lines().count(),
                            deleted_lines: 0,
                        },
                    );
                }
                (Some(from), None) => {
                    delta.add_file(
                        path.clone(),
                        FileDelta {
                            delta_type: DeltaType::Deleted,
                            old_path: Some(path.clone()),
                            new_path: None,
                            old_hash: Some(from.object_hash.clone()),
                            new_hash: None,
                            diff: None,
                            added_lines: 0,
                            deleted_lines: 0,
                        },
                    );
                }
                (Some(from), Some(to)) if from.object_hash != to.object_hash => {
                    let old_blob = Blob::load(&from.object_hash, &objects_dir)?;
                    let new_blob = Blob::load(&to.object_hash, &objects_dir)?;
                    let (diff, added, deleted) = text_diff(
                        &String::from_utf8_lossy(&old_blob.data),
                        &String::from_utf8_lossy(&new_blob.data),
                    );

                    delta.add_file(
                        path.clone(),
                        FileDelta {
                            delta_type: DeltaType::Modified,
                            old_path: Some(path.clone()),
                            new_path: Some(path.clone()),
                            old_hash: Some(from.object_hash.clone()),
                            new_hash: Some(to.object_hash.clone()),
                            diff: Some(diff),
                            added_lines: added,
                            deleted_lines: deleted,
                        },
                    );
                }
                _ => {}
            }
        }

        let mut changes = Vec::new();

        let deleted_files: Vec<_> = delta
            .filter_by_type(DeltaType::Deleted)
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let added_files: Vec<_> = delta
            .filter_by_type(DeltaType::Added)
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (deleted_path, deleted_delta) in deleted_files {
            if let Some((added_path, added_delta)) = added_files
                .iter()
                .find(|(_, ad)| ad.new_hash == deleted_delta.old_hash)
            {
                changes.push((
                    deleted_path.clone(),
                    added_path.clone(),
                    added_delta.clone(),
                ));
            }
        }

        for (deleted_path, added_path, added_delta) in changes {
            delta.files.remove(&deleted_path);
            delta.files.remove(&added_path);

            delta.add_file(
                added_path.to_path_buf(),
                FileDelta {
                    delta_type: DeltaType::Renamed,
                    old_path: Some(deleted_path),
                    new_path: Some(added_path.to_path_buf()),
                    old_hash: added_delta.old_hash.clone(),
                    new_hash: added_delta.new_hash.clone(),
                    diff: None,
                    added_lines: 0,
                    deleted_lines: 0,
                },
            );
        }

        Ok(delta)
    }
}

impl VoxObject for Tree {
    fn object_type(&self) -> &str {
        "tree"
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = Vec::new();

        for entry in &self.entries {
            let mode_and_name = format!("{} {}\0", entry.permissions, entry.name);
            content.extend_from_slice(mode_and_name.as_bytes());

            let hash_bytes = hex::decode(&entry.object_hash).expect("Decoding failed");
            content.extend_from_slice(&hash_bytes);
        }
        Ok(content)
    }

    fn hash(&self) -> Result<String> {
        let content = self.serialize()?;
        let mut hasher = Sha1::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn object_path(&self) -> Result<String> {
        let hash = self.hash()?;
        Ok(format!(
            "{}/{}/{}",
            OBJ_DIR.display(),
            &hash[0..2],
            &hash[2..]
        ))
    }
}

pub fn create_tree(path: &Path) -> Result<Tree> {
    let mut tree = Tree {
        entries: Vec::new(),
    };

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 file name"))?
            .to_string();

        if name.starts_with('.') || name == "target" {
            continue;
        }

        if entry_path.is_file() {
            let blob = Blob::new(entry_path.to_str().context("Invalid file path")?)?;
            let object_hash = blob.save(&PathBuf::from(&*OBJ_DIR))?;
            tree.entries.push(TreeEntry {
                object_type: "blob".to_string(),
                permissions: "100644".to_string(),
                object_hash,
                name,
            });
        } else if entry_path.is_dir() {
            let subtree = create_tree(&entry_path)?;
            if !subtree.entries.is_empty() {
                let hash_str = store_tree(&subtree)?;
                tree.entries.push(TreeEntry {
                    object_type: "tree".to_string(),
                    permissions: "40000".to_string(),
                    object_hash: hash_str,
                    name,
                });
            }
        }
    }

    tree.entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tree)
}

pub fn store_tree(tree: &Tree) -> Result<String> {
    let content = tree.serialize()?;
    let header = format!("tree {}\0", content.len());
    let full_content = [header.as_bytes(), &content].concat();

    let mut hasher = Sha1::new();
    hasher.update(&full_content);
    let hash = format!("{:x}", hasher.finalize());

    let object_path = PathBuf::from(&*OBJ_DIR).join(&hash[..2]).join(&hash[2..]);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&full_content)?;
    let compressed = encoder.finish()?;

    if !object_path.exists() {
        fs::create_dir_all(object_path.parent().context("Invalid object path")?)?;
        fs::write(&object_path, compressed)?;
    }

    Ok(hash)
}

pub fn read_tree(hash: &str, objects_dir: &PathBuf) -> Result<Tree> {
    let object_path = objects_dir.join(&hash[..2]).join(&hash[2..]);

    let compressed = fs::read(&object_path)?;
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;

    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .context("Invalid format: no null byte found")?;

    let content = &data[null_pos + 1..];
    let mut entries = Vec::new();
    let mut pos = 0;

    while pos < content.len() {
        let null_pos = content[pos..]
            .iter()
            .position(|&b| b == 0)
            .context("Invalid format: no null byte found in entry")?;

        let entry_meta = std::str::from_utf8(&content[pos..pos + null_pos])?;
        let (permissions, name) = entry_meta
            .split_once(' ')
            .context("Invalid format: no space in entry metadata")?;

        pos += null_pos + 1;

        let hash_bytes = &content[pos..pos + 20];
        let object_hash = hex::encode(hash_bytes);
        pos += 20;

        entries.push(TreeEntry {
            permissions: permissions.to_string(),
            object_type: if permissions.starts_with("40") {
                "tree".to_string()
            } else {
                "blob".to_string()
            },
            object_hash,
            name: name.to_string(),
        });
    }

    Ok(Tree { entries })
}
