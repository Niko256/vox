// checkout_phase.rs
use super::clone::CloneCommand;
use crate::storage::objects::tree::Tree;
use crate::storage::objects::Object;
use crate::storage::repo::Repository;
use crate::storage::utils::{OBJ_TYPE_BLOB, OBJ_TYPE_TREE};
use anyhow::{anyhow, bail, Result};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use tokio::fs;

impl CloneCommand {
    pub async fn checkout_workdir(
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

        self.checkout_tree(repo.workdir(), tree, objects).await
    }

    async fn checkout_tree(
        &self,
        root_path: &Path,
        root_tree: &Tree,
        objects: &HashMap<String, Object>,
    ) -> Result<()> {
        let mut stack = VecDeque::new();
        stack.push_back((root_path.to_path_buf(), root_tree));

        while let Some((current_path, tree)) = stack.pop_front() {
            fs::create_dir_all(&current_path).await?;

            for entry in &tree.entries {
                let entry_path = current_path.join(&entry.name);

                match entry.object_type.as_str() {
                    OBJ_TYPE_BLOB => {
                        let blob = match objects.get(&entry.object_hash) {
                            Some(Object::Blob(b)) => b,
                            _ => bail!("Missing blob {}", entry.object_hash),
                        };
                        fs::write(entry_path, &blob.data).await?;
                    }
                    OBJ_TYPE_TREE => {
                        let subtree = match objects.get(&entry.object_hash) {
                            Some(Object::Tree(t)) => t,
                            _ => bail!("Missing tree {}", entry.object_hash),
                        };
                        stack.push_back((entry_path, subtree));
                    }
                    _ => bail!("Unsupported object type in tree"),
                }
            }
        }

        Ok(())
    }
}
