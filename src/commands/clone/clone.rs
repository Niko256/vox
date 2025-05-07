use crate::commands::config::config::Config;
use crate::connection::VoxTransport;
use crate::storage::objects::delta::apply_delta;
use crate::storage::objects::pack::{PackObject, Packfile};
use crate::storage::objects::{Object, ObjectStore, VoxObject};
use crate::storage::refs::{read_ref, write_ref};
use crate::storage::repo::Repository;
use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io;
use url::Url;

pub(crate) struct CloneCommand {
    url: Url,
    target: PathBuf,
    config: Option<Config>,
}

impl CloneCommand {
    pub fn new(url: Url, target: PathBuf, config: Option<Config>) -> Self {
        Self {
            url,
            target,
            config,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        let repo = Repository::new_remote("origin", &self.target, self.url.clone());

        let transport = VoxTransport::new(self.url.as_str())?;

        let refs = self.fetch_refs(&transport).await?;

        let packfile = self.fetch_packfile(&transport, &refs).await?;

        let objects = self.reconstruct_objects(packfile)?;

        self.write_refs(&repo, &refs).await?;

        Ok(())
    }

    async fn fetch_refs(&self, transport: &VoxTransport) -> Result<HashMap<String, String>> {
        let server_refs = transport.list_refs().await?;
        let mut refs = HashMap::new();

        for r in server_refs {
            refs.insert(r.name, r.hash);
        }

        Ok(refs)
    }

    async fn fetch_packfile(
        &self,
        transport: &VoxTransport,
        refs: &HashMap<String, String>,
    ) -> Result<Packfile> {
        let want: Vec<String> = refs.values().cloned().collect();
        let pack_data = transport.fetch_packfile(&want).await?;
        Packfile::deserialize(&pack_data)
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
}

pub async fn clone_command(
    url: impl AsRef<str>,
    target_dir: impl AsRef<Path>,
    config: Option<Config>,
) -> Result<()> {
    let url = Url::parse(url.as_ref())?;
    let target_dir = target_dir.as_ref().to_path_buf();

    let cmd = CloneCommand::new(url, target_dir, config);
    cmd.execute().await
}
