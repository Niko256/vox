use super::blob::Blob;
use super::delta::{Delta, DeltaType};
use crate::commands::diff::diff::text_diff;
use crate::storage::objects::{delta::DiffSummary, Loadable, Storable, VoxObject};
use crate::storage::utils::{OBJ_DIR, OBJ_TYPE_BLOB, OBJ_TYPE_TREE, PERM_DIR, PERM_FILE};
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Represents a single entry in a tree object
#[derive(Debug)]
pub struct TreeEntry {
    /// Unix file permissionds in string format (e.g., "100644" for regular files)
    pub mode: String,
    /// Type of the object
    pub object_type: String,
    /// SHA-1 hash of the referenced object
    pub object_hash: String,
    /// Name of the file or directory
    pub name: String,
}

/// Represents a directory tree
///
/// A Tree object contains a list of entries representing files and subdirectories
#[derive(Debug)]
pub struct Tree {
    /// List of entries in this tree
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    /// Compares two trees and generates a Delta describing all changes between them
    ///
    /// This is the main entry point that orchestrates the comparison process by:
    /// 1. Collecting all paths from both trees
    /// 2. Comparing corresponding entries
    /// 3. Detecting file renames
    ///
    /// # Arguments
    ///
    /// * `from` - Source tree to compare from
    /// * `to` - Target tree to compare to
    /// * `objects_dir` - Path to the objects directory
    ///
    /// # Returns
    ///
    /// Returns a [`Delta`] containing all changes or an error if comparison fails
    pub fn compare_trees(from: &Tree, to: &Tree, objects_dir: &Path) -> Result<Delta> {
        let mut delta = Delta::new(from.hash().ok(), to.hash().ok());
        let all_paths = Self::collect_all_paths(from, to);
        Self::compare_entries(&mut delta, from, to, &all_paths, objects_dir)?;
        Self::detect_renames(&mut delta, objects_dir)?;
        Ok(delta)
    }

    /// Collects all unique paths from both source and target trees
    ///
    /// This creates a unified view of all paths that need to be compared,
    /// regardless of whether they exist in one tree or both
    ///
    /// # Arguments
    ///
    /// * `from` - Source tree
    /// * `to` - Target tree
    ///
    /// # Returns
    ///
    /// Returns a [`HashSet`] containing all unique paths from both trees
    fn collect_all_paths(from: &Tree, to: &Tree) -> HashSet<PathBuf> {
        let mut paths = HashSet::new();
        for entry in &from.entries {
            paths.insert(PathBuf::from(&entry.name));
        }
        for entry in &to.entries {
            paths.insert(PathBuf::from(&entry.name));
        }
        paths
    }

    /// Compares corresponding entries across all paths in both trees
    ///
    /// For each path, determines what change occurred (addition, deletion, modification)
    /// and records it in the Delta
    ///
    /// # Arguments
    ///
    /// * `delta` - Mutable reference to Delta being constructed
    /// * `from` - Source tree
    /// * `to` - Target tree
    /// * `all_paths` - All paths to compare
    /// * `objects_dir` - Path to objects directory
    ///
    /// # Errors
    ///
    /// Returns an error if any object fails to load during comparison
    fn compare_entries(
        delta: &mut Delta,
        from: &Tree,
        to: &Tree,
        all_paths: &HashSet<PathBuf>,
        objects_dir: &Path,
    ) -> Result<()> {
        let from_entries: HashMap<&str, &TreeEntry> =
            from.entries.iter().map(|e| (e.name.as_str(), e)).collect();

        let to_entries: HashMap<&str, &TreeEntry> =
            to.entries.iter().map(|r| (r.name.as_str(), r)).collect();

        for path_buf in all_paths {
            let path_str = path_buf.to_str().context("Path contains invalid UTF-8")?;

            let from_entry = from_entries.get(path_str);
            let to_entry = to_entries.get(path_str);

            Self::process_entry_pair(
                delta,
                path_buf,
                from_entry.copied(),
                to_entry.copied(),
                objects_dir,
            )?;
        }

        Ok(())
    }

    /// Processes a pair of corresponding entries from old and new trees
    ///
    /// Determines the type of change (if any) between the entries and handles it
    /// appropriately
    ///
    /// # Arguments
    ///
    /// * `delta` - Mutable reference to Delta
    /// * `path` - Current path being processed
    /// * `from_entry` - Optional entry from source tree
    /// * `to_entry` - Optional entry from target tree
    /// * `objects_dir` - Path to objects directory
    ///
    /// # Errors
    ///
    /// Returns an error if blob contents fail to load during diff calculation
    fn process_entry_pair(
        delta: &mut Delta,
        path: &PathBuf,
        from_entry: Option<&TreeEntry>,
        to_entry: Option<&TreeEntry>,
        objects_dir: &Path,
    ) -> Result<()> {
        match (from_entry, to_entry) {
            (None, Some(to)) => Self::handle_added(delta, path, to),
            (Some(from), None) => Self::handle_deleted(delta, path, from),
            (Some(from), Some(to)) if from.object_hash != to.object_hash => {
                Self::handle_modified(delta, path, from, to, objects_dir)
            }
            _ => Ok(()),
        }
    }

    /// Records an added file in the Delta
    ///
    /// # Arguments
    ///
    /// * `delta` - Mutable reference to Delta
    /// * `path` - Path where addition occurred
    /// * `to` - New tree entry
    fn handle_added(delta: &mut Delta, path: &PathBuf, to: &TreeEntry) -> Result<()> {
        delta.add_change(DeltaType::ADDED {
            path: path.clone(),
            new_hash: to.object_hash.clone(),
        });
        Ok(())
    }

    /// Records a deleted file in the Delta
    ///
    /// # Arguments
    ///
    /// * `delta` - Mutable reference to Delta
    /// * `path` - Path where deletion occurred
    /// * `from` - Removed tree entry
    ///
    fn handle_deleted(delta: &mut Delta, path: &PathBuf, from: &TreeEntry) -> Result<()> {
        delta.add_change(DeltaType::DELETED {
            path: path.clone(),
            old_hash: from.object_hash.clone(),
        });
        Ok(())
    }

    /// Records a modified file in the Delta
    ///
    /// For blob files, calculates detailed diff summary including:
    /// - Number of insertions
    /// - Number of deletions
    /// - Unified diff format text
    ///
    /// # Arguments
    ///
    /// * `delta` - Mutable reference to Delta
    /// * `path` - Path where modification occurred
    /// * `from` - Original version
    /// * `to` - Modified version
    /// * `objects_dir` - Path to objects directory
    ///
    /// # Errors
    ///
    /// Returns an error if blob contents fail to load
    fn handle_modified(
        delta: &mut Delta,
        path: &PathBuf,
        from: &TreeEntry,
        to: &TreeEntry,
        objects_dir: &Path,
    ) -> Result<()> {
        let summary = if from.object_type == OBJ_TYPE_BLOB && to.object_type == OBJ_TYPE_BLOB {
            Self::calculate_diff_summary(&from.object_hash, &to.object_hash, objects_dir)?
        } else {
            None
        };

        delta.add_change(DeltaType::MODIFIED {
            path: path.clone(),
            old_hash: from.object_hash.clone(),
            new_hash: to.object_hash.clone(),
            summary,
        });
        Ok(())
    }

    /// Calculates detailed diff between two blob objects
    ///
    /// Uses Myers diff algorithm to compute:
    /// - Insertion count
    /// - Deletion count
    /// - Unified diff text
    ///
    /// # Arguments
    ///
    /// * `old_hash` - Hash of original blob
    /// * `new_hash` - Hash of modified blob
    /// * `objects_dir` - Path to objects directory
    ///
    /// # Returns
    ///
    /// Returns [`Option<DiffSummary>`] with diff details if blobs are text files,
    /// or None for binary files or errors
    fn calculate_diff_summary(
        old_hash: &str,
        new_hash: &str,
        objects_dir: &Path,
    ) -> Result<Option<DiffSummary>> {
        let old_blob = Blob::load(old_hash, objects_dir)?;
        let new_blob = Blob::load(new_hash, objects_dir)?;
        let (text_diff, insertions, removals) = text_diff(
            &String::from_utf8_lossy(&old_blob.data),
            &String::from_utf8_lossy(&new_blob.data),
        );

        Ok(Some(DiffSummary::new(
            insertions,
            removals,
            Some(text_diff),
        )))
    }

    /// Detects file renames by matching deleted and added files with identical content
    ///
    /// Scans the Delta for matching hash pairs between deletions and additions,
    /// converting them to rename operations
    ///
    /// # Arguments
    ///
    /// * `delta` - Mutable reference to Delta being analyzed
    /// * `objects_dir` - Path to objects directory (unused in current implementation)
    ///
    /// # Errors
    ///
    /// Currently always returns Ok, but signature maintained for future error cases
    fn detect_renames(delta: &mut Delta, _objects_dir: &Path) -> Result<()> {
        let (deleted, added) = Self::collect_deleted_and_added(delta);
        let renames = Self::find_rename_candidates(&deleted, &added)?;

        for (old_path, new_path, hash) in renames {
            delta.get().remove(&old_path);
            delta.get().remove(&new_path);

            delta.add_change(DeltaType::RENAMED {
                old_path,
                new_path,
                old_hash: hash.clone(),
                new_hash: hash,
                summary: None,
            });
        }
        Ok(())
    }

    /// Collects all deleted and added files from Delta, indexed by their content hash
    ///
    /// # Arguments
    ///
    /// * `delta` - Reference to Delta being analyzed
    ///
    /// # Returns
    ///
    /// Returns a tuple of HashMaps:
    /// - First map: deleted files (hash -> path)
    /// - Second map: added files (hash -> path)
    fn collect_deleted_and_added(
        delta: &Delta,
    ) -> (HashMap<String, PathBuf>, HashMap<String, PathBuf>) {
        let mut deleted = HashMap::new();
        let mut added = HashMap::new();

        for (_, dt) in &delta.get() {
            match dt {
                DeltaType::DELETED { path, old_hash } => {
                    deleted.insert(old_hash.clone(), path.clone());
                }
                DeltaType::ADDED { path, new_hash } => {
                    added.insert(new_hash.clone(), path.clone());
                }
                _ => {}
            }
        }

        (deleted, added)
    }

    /// Identifies potential file renames by matching hashes between deletions and additions
    ///
    /// # Arguments
    ///
    /// * `deleted` - Map of deleted file hashes to paths
    /// * `added` - Map of added file hashes to paths
    ///
    /// # Returns
    ///
    /// Returns a vector of rename candidates as tuples:
    /// (old_path, new_path, content_hash)
    fn find_rename_candidates(
        deleted: &HashMap<String, PathBuf>,
        added: &HashMap<String, PathBuf>,
    ) -> Result<Vec<(PathBuf, PathBuf, String)>> {
        let mut candidates = Vec::new();

        for (hash, del_path) in deleted {
            if let Some(add_path) = added.get(hash) {
                candidates.push((del_path.clone(), add_path.clone(), hash.clone()));
            }
        }

        Ok(candidates)
    }
}

/// Creates a Tree object representing the directory structure at the given path
///
/// # Arguments
///
/// * `path` - The filesystem path to scan
///
/// # Errors
///
/// Returns an error if the directory cannot be read or any files cannot be processed
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

        // Skip hidden files and target directories
        if name.starts_with('.') || name == "target" {
            continue;
        }

        if entry_path.is_file() {
            // Create blob for file
            let blob = Blob::new(entry_path.to_str().context("Invalid file path")?)?;
            let object_hash = blob.save(&PathBuf::from(&*OBJ_DIR))?;
            tree.entries.push(TreeEntry {
                object_type: OBJ_TYPE_BLOB.to_string(),
                mode: PERM_FILE.to_string(), // Regular file mode
                object_hash,
                name,
            });
        } else if entry_path.is_dir() {
            // Recursively create subtree
            let subtree = create_tree(&entry_path)?;
            if !subtree.entries.is_empty() {
                let hash_str = store_tree(&subtree)?;
                tree.entries.push(TreeEntry {
                    object_type: OBJ_TYPE_TREE.to_string(),
                    mode: PERM_DIR.to_string(), // Directory mode
                    object_hash: hash_str,
                    name,
                });
            }
        }
    }

    // Sort entries by name for consistent hashing
    tree.entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tree)
}

/// Stores a tree object in the object database
///
/// # Arguments
///
/// * `tree` - The tree to store
///
/// # Returns
///
/// The SHA-1 hash of the stored tree
///
pub fn store_tree(tree: &Tree) -> Result<String> {
    let content = tree.serialize()?;
    let header = format!("tree {}\0", content.len());
    let full_content = [header.as_bytes(), &content].concat();

    // Compute hash
    let mut hasher = Sha1::new();
    hasher.update(&full_content);
    let hash = format!("{:x}", hasher.finalize());

    // Create object path
    let object_path = PathBuf::from(&*OBJ_DIR).join(&hash[..2]).join(&hash[2..]);

    // Compress and write if not exists
    if !object_path.exists() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_content)?;
        let compressed = encoder.finish()?;

        fs::create_dir_all(object_path.parent().context("Invalid object path")?)?;
        fs::write(&object_path, compressed)?;
    }

    Ok(hash)
}

/// Reads a tree object from the object database
///
/// # Arguments
///
/// * `hash` - The SHA-1 hash of the tree to read
/// * `objects_dir` - Path to the objects directory
///
pub fn read_tree(hash: &str, objects_dir: &Path) -> Result<Tree> {
    let object_path = objects_dir.join(&hash[..2]).join(&hash[2..]);

    // Read and decompress object
    let compressed = fs::read(&object_path)?;
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;

    // Parse header
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .context("Invalid format: no null byte found")?;

    // Parse entries
    let content = &data[null_pos + 1..];
    let mut entries = Vec::new();
    let mut pos = 0;

    while pos < content.len() {
        // Parse entry metadata (mode and name)
        let null_pos = content[pos..]
            .iter()
            .position(|&b| b == 0)
            .context("Invalid format: no null byte found in entry")?;

        let entry_meta = std::str::from_utf8(&content[pos..pos + null_pos])?;
        let (mode, name) = entry_meta
            .split_once(' ')
            .context("Invalid format: no space in entry metadata")?;

        pos += null_pos + 1;

        // Parse object hash
        let hash_bytes = &content[pos..pos + 20];
        let object_hash = hex::encode(hash_bytes);
        pos += 20;

        // Determine object type from mode
        let object_type = if mode.starts_with("40") {
            OBJ_TYPE_TREE.to_string()
        } else {
            OBJ_TYPE_BLOB.to_string()
        };

        entries.push(TreeEntry {
            mode: mode.to_string(),
            object_type,
            object_hash,
            name: name.to_string(),
        });
    }

    Ok(Tree { entries })
}

impl VoxObject for Tree {
    fn object_type(&self) -> &str {
        OBJ_TYPE_TREE
    }

    /// Serializes the tree to bytes
    ///
    /// The format is: `[mode] [name]\0[20-byte hash]` for each entry
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = Vec::new();

        for entry in &self.entries {
            let mode_and_name = format!("{} {}\0", entry.mode, entry.name);
            content.extend_from_slice(mode_and_name.as_bytes());

            let hash_bytes = hex::decode(&entry.object_hash).expect("Decoding failed");
            content.extend_from_slice(&hash_bytes);
        }
        Ok(content)
    }

    /// Computes the SHA-1 hash of the serialized tree
    fn hash(&self) -> Result<String> {
        let content = self.serialize()?;
        let mut hasher = Sha1::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Returns the storage path for this tree in the objects directory
    ///
    /// The path follows Git's convention: `objects/xx/yyyy...` where xx is the
    /// first two hex digits of the hash and yyyy... is the rest
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

impl Loadable for Tree {
    fn load(hash: &str, objects_dir: &Path) -> Result<Self> {
        read_tree(hash, objects_dir)
    }
}
