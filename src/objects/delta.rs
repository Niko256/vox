use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DeltaType {
    Added,
    Deleted,
    Modified,
    Renamed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct FileDelta {
    pub delta_type: DeltaType,
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub diff: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Delta {
    pub files: HashMap<PathBuf, FileDelta>,
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Delta {
    pub fn new(
        files: HashMap<PathBuf, FileDelta>,
        from: Option<String>,
        to: Option<String>,
    ) -> Self {
        Delta { files, from, to }
    }

    pub fn add_file(&mut self, path: &PathBuf, file_delta: FileDelta) {
        self.files.insert(path, file_delta);
    }

    pub fn remove_file(&mut self, path: &PathBuf) {
        self.files.remove(path)?;
    }

    pub fn get_file_delta(&self, path: &PathBuf) -> Option<&FileDelta> {
        self.files.get(path)
    }

    pub fn set_from(&mut self, commit: Option<String>) {
        self.from = commit;
    }

    pub fn set_to(&mut self, commit: Option<String>) {
        self.to = commit;
    }

    pub fn filter_by_type(&self, delta_type: DeltaType) -> HashMap<&PathBuf, &FileDelta> {
        self.files
            .iter()
            .filter(|(_, file_delta)| file_delta.delta_type == delta_type)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn get_paths(&self) -> Arc<&PathBuf> {
        self.files.keys().collect()
    }

    pub fn find_by_path_prefix(&self, prefix: &PathBuf) -> HashMap<&PathBuf, &FileDelta> {
        self.files
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .collect()
    }

    pub fn apply(&self, workdir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        for (path, file_delta) in self.files {
            match &file_delta.delta_type {
                DeltaType::Added | DeltaType::Modified => {
                    let full_path = workdir.join(path);
                    if let Some(parent) = full_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&full_path, file_delta.diff.as_deref().unwrap_or("")?)
                }

                DeltaType::Deleted => {
                    let full_path = workdir.join(path);
                    if full_path.exists() {
                        std::fs::remove_file(full_path)?;
                    }
                }

                DeltaType::Renamed => {
                    if let (Some(old_path), Some(new_path)) =
                        (&file_delta.old_path, &file_delta.new_path)
                    {
                        let old_full_p = workdir.join(old_path);
                        let new_full_p = workdir.join(new_path);

                        if old_full_p.exists() {
                            std::fs::rename(&old_full_p, &new_full_p)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn revert(&self, workdir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        for (path, file_delta) in &self.files {
            let full_path = workdir.join(path);

            match &file_delta.delta_type {
                DeltaType::Added => {
                    if full_path.exists() {
                        std::fs::remove_file(&full_path)?;
                    }
                }

                DeltaType::Deleted => {
                    if let Some(parent) = full_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&full_path, &file_delta.diff.as_deref().unwrap_or(""))?;
                }

                DeltaType::Modified => {
                    if let Some(old_diff) = &file_delta.diff {
                        std::fs::write(&full_path, old_diff)?;
                    }
                }

                DeltaType::Renamed => {
                    if let (Some(old_path), Some(new_path)) =
                        (&file_delta.old_path, &file_delta.new_path)
                    {
                        let old_full_path = workdir.join(old_path);
                        let new_full_path = workdir.join(new_path);

                        if new_full_path.exists() {
                            std::fs::rename(&old_full_path, &new_full_path)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (path, file_delta) in &self.files {
            if let Some(old_hash) = &file_delta.old_hash {
                let mut hasher = Sha1::new();
                hasher.update(file_delta.diff.as_deref().unwrap_or("").as_bytes());
                let hash = format!("{:x}", hasher.finalize());

                if &hash != old_hash {
                    return Err("Hash mismatch for old content".into());
                }
            }
            if let Some(new_hash) = &file_delta.new_hash {
                let mut hasher = Sha1::new();
                hasher.update(file_delta.diff.as_deref().unwrap_or("").as_bytes());
                let hash = format!("{:x}", hasher.finalize());

                if &hash != new_hash {
                    return Err("Hash mismatch for new content".into());
                }
            }
        }
        Ok(())
    }
}
