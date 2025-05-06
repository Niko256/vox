use crate::commands::branch::checkout::checkout_command;
use crate::storage::objects::tree::Tree;
use crate::storage::objects::{self, Object};
use crate::storage::repo::Repository;
use crate::storage::utils::{HEAD_DIR, REFS_DIR};
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::clone::CloneCommand;

impl CloneCommand {
    fn checkout_working_dir(
        &self,
        repo: &Repository,
        objects: &HashMap<String, Object>,
        refs: &HashMap<String, String>,
    ) -> Result<()> {
        let head_commit_hash = refs
            .get("HEAD")
            .ok_or_else(|| anyhow!("No HEAD reference found"))?;

        let commit = match objects.get(head_commit_hash) {
            Some(Object::Commit(c)) => c,
            _ => bail!("HEAD reference points to non-commit object"),
        };

        let tree = match objects.get(&commit.tree) {
            Some(Object::Tree(t)) => t,
            _ => bail!("Commit tree not found"),
        };

        self.checkout_tree(repo.workdir(), tree, objects)?;

        Ok(())
    }

    fn checkout_tree(
        &self,
        path: &Path,
        tree: &Tree,
        objects: &HashMap<String, Object>,
    ) -> Result<()> {
        std::fs::create_dir_all(path)?;

        for entry in &tree.entries {
            let entry_path = path.join(&entry.name);

            match entry.object_type.as_str() {
                "blob" => {
                    let blob = match objects.get(&entry.object_hash) {
                        Some(Object::Blob(b)) => b,
                        _ => bail!("Missing blob {}", entry.object_hash),
                    };
                    std::fs::write(entry_path, &blob.data)?;
                }
                "tree" => {
                    let subtree = match objects.get(&entry.object_hash) {
                        Some(Object::Tree(t)) => t,
                        _ => bail!("Missing tree {}", entry.object_hash),
                    };
                    self.checkout_tree(&entry_path, subtree, objects)?;
                }
                _ => bail!("Unsupported object type in tree"),
            }
        }

        Ok(())
    }
}
