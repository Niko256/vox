use crate::commands::config::config::Config;
use crate::commands::init::init::init_command;
use crate::connection::VoxTransport;
use crate::storage::objects::delta::{apply_delta, Delta};
use crate::storage::objects::pack::{PackObject, Packfile};
use crate::storage::objects::{Object, ObjectStorage, VoxObject};
use crate::storage::refs::write_ref;
use crate::storage::repo::Repository;
use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;
use log;

pub(crate) struct CloneCommand {
    url: Url,
    target: PathBuf,
    repo_name: String,
    config: Option<Config>,
}

impl CloneCommand {
    pub fn new(
        url: Url,
        target: impl Into<PathBuf>,
        repo_name: impl Into<String>,
        config: Option<Config>,
    ) -> Result<Self> {

        if !url.has_host() {
            bail!("Invalid URL: host is required!");
        }

        Ok(Self {
            url,
            target: target.into(),
            repo_name: repo_name.into(),
            config,
        })
    }


    pub async fn try_execute(&self) -> Result<()> {
        if Repository::is_initialized(&self.target).await? {
            bail!("Target directory already contains a vox repository");
        }
        
        println!("Cloning into '{}'...", self.target.display());

        let cleanup_on_error = || async {
            if let Err(e) = tokio::fs::remove_dir_all(&self.target).await {
                log::warn!("Failed to cleanup target dir: {}", e);
            }
        };
        
        if let Err(e) = self.execute().await {
            cleanup_on_error().await;
            return Err(e);
        }
        Ok(())
    }


    async fn execute(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.target).await?;
        init_command().await.context("Failed to initialize repo")?;

        let repo = Repository::new_remote(&self.repo_name, &self.target, self.url.clone());
        let transport = VoxTransport::new(self.url.as_str())?;

        println!("Fetching references...");

        let refs = self.fetch_refs(&transport).await?;

        println!("Fetching objects...");
        let packfile = self.fetch_packfile(&transport, &refs).await?;
        let delta_cnt = packfile.objects.iter().filter(|o| matches!(o, PackObject::Delta { .. }))
            .count();

        println!("Reconstructing objects...");
        let objects = self.reconstruct_objects(packfile)?;
        println!("Resolved {} deltas", delta_cnt);


        println!("Writing objects...");
        self.save_objects(&repo, &objects).await?;

        println!("Updating references...");
        self.write_refs(&repo, &refs).await?;

        println!("Checking out files...");
        self.checkout_workdir(&repo, &objects, &refs).await?;

        println!("Clone of '{}' completed successfully!", self.url);
        Ok(())
    }


    fn reconstruct_objects(&self, packfile: Packfile) -> Result<HashMap<String, Object>> {
        let mut base_objects = HashMap::new();
        let mut all_objects = HashMap::new();

        for obj in &packfile.objects {
            if let PackObject::Base(data, obj_type) = obj {
                let obj = Packfile::parse_object(*obj_type, data)?;
                let hash = obj.hash()?;
                base_objects.insert(hash.clone(), data.clone());
                all_objects.insert(hash, obj);
            }
        }

        for obj in &packfile.objects {
            if let PackObject::Delta { base_hash, data } = obj {
                let base_data = base_objects
                    .get(base_hash)
                    .ok_or_else(|| anyhow!("Missing base object {}", base_hash))?;

                let base_size = base_data.len();
                let mut delta = Delta::new(data);
                let (expected_base_size, _) = delta.parse_header()?;

                if base_size != expected_base_size {
                    bail!("Base size mismatch (expected {}, got {})", expected_base_size, base_size);
                }

                let reconstructed = apply_delta(base_data, data)?;
                let obj_type = Packfile::detect_type(&reconstructed)?;
                let obj = Packfile::parse_object(obj_type, &reconstructed)?;
                let hash = obj.hash()?;
                all_objects.insert(hash, obj);
            }
        }

        Ok(all_objects)
    }

    async fn write_refs(&self, repo: &Repository, refs: &HashMap<String, String>) -> Result<()> {
        let refs_dir = repo.workdir().join("refs").join("remotes").join("origin");

        for (ref_name, commit_hash) in refs {
            write_ref(&refs_dir, ref_name, commit_hash)
                .await
                .with_context(|| format!("Failed to write ref {}", ref_name))?;
        }

        if let Some(main_branch) = refs.get("HEAD") {
            write_ref(repo.workdir(), "HEAD", main_branch).await?;
        }

        Ok(())
    }

    async fn save_objects(
        &self,
        repo: &Repository,
        objects: &HashMap<String, Object>,
    ) -> Result<()> {
        let storage = ObjectStorage::new(repo.workdir());

        for (_, obj) in objects {
            let path = storage.dir.join(obj.object_path()?);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(path, obj.serialize()?).await?;
        }

        Ok(())
    }
}

pub async fn clone_command(
    url: impl AsRef<str>,
    target_dir: impl AsRef<Path>,
    repo_name: impl Into<String>,
    config: Option<Config>,
) -> Result<()> {
    let url = Url::parse(url.as_ref())?;
    let target_dir = target_dir.as_ref().to_path_buf();
    let repo_name = repo_name.into();

    let cmd = CloneCommand::new(url, target_dir, repo_name, config)?;
    cmd.try_execute().await
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::init::init::init_command,
        storage::objects::{blob::Blob, commit::Commit, tree::{Tree, TreeEntry}},
    };
    use tempfile::tempdir;
    use crate::storage::repo::Repository;
    use chrono::Utc;

    async fn create_test_repo() -> (tempfile::TempDir, Repository) {
        let dir = tempdir().unwrap();
        let repo = Repository::new_local("test-repo", dir.path());
        init_command().await.unwrap();
        (dir, repo)
    }

    fn create_test_objects() -> (HashMap<String, Object>, HashMap<String, String>) {
        let mut objects = HashMap::new();
        let mut refs = HashMap::new();

        let blob = Blob { data: b"test content".to_vec() };
        let blob_hash = blob.hash().unwrap();
        objects.insert(blob_hash.clone(), Object::Blob(blob));

        let mut tree = Tree { entries: vec![] };
        tree.entries.push(TreeEntry {
            name: "test.txt".to_string(),
            object_hash: blob_hash,
            object_type: "blob".to_string(),
            mode: "100644".to_string(),
        });
        let tree_hash = tree.hash().unwrap();
        objects.insert(tree_hash.clone(), Object::Tree(tree));

        let timestamp = Utc::now(); 

        let commit = Commit {
            tree: tree_hash,
            parent: None,
            author: "test".to_string(),
            message: "test commit".to_string(),
            timestamp,
        };
        let commit_hash = commit.hash().unwrap();
        objects.insert(commit_hash.clone(), Object::Commit(commit));

        refs.insert("HEAD".to_string(), commit_hash);

        (objects, refs)
    }


    #[tokio::test]
    async fn test_delta_reconstruction() {
        let (dir, _) = create_test_repo().await;
        
        let mut packfile = Packfile::new();
        
        let base_blob = Blob { data: b"base content".to_vec() }; // 12 bytes

        packfile.add_object(&base_blob).unwrap();
        
        let delta_data = vec![0x10, 0x00, 0x00, 0x01, 0x0A];

        packfile.objects.push(PackObject::Delta {
            base_hash: base_blob.hash().unwrap(),
            data: delta_data,
        });

        let cmd = CloneCommand::new(
            Url::parse("http://example.com/repo.git").unwrap(),
            dir.path().join("delta-test"),
            "delta-test",
            None,
        ).unwrap();

        
        let result = cmd.reconstruct_objects(packfile);

        assert!(result.is_err(), "Expected reconstruct_objects to fail");

        let error_message = match result {
            Ok(_) => panic!("Expected reconstruct_objects to fail"),
            Err(e) => e.to_string(),
        };

        println!("Received error: {}", error_message);
        assert!(error_message.contains("Base size mismatch"), "Error message should contain 'Base size mismatch'");
        assert!(error_message.contains("expected 16, got 12"), "Error message should contain expected/got sizes");


        assert!(!dir.path().join("delta-test").exists());
    }
}
