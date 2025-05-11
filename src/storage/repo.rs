use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::{fs, io};
use url::Url;

use crate::storage::utils::{HEAD_DIR, OBJ_DIR, REFS_DIR, VOX_DIR};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepoType {
    Local,
    Remote {
        #[serde(serialize_with = "serialize_url", deserialize_with = "deserialize_url")]
        url: Url,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub workdir: PathBuf,
    pub repo_type: RepoType,
}

impl Repository {
    pub fn new_local(name: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            workdir: workdir.into(),
            repo_type: RepoType::Local,
        }
    }

    pub fn new_remote(name: impl Into<String>, workdir: impl Into<PathBuf>, url: Url) -> Self {
        Self {
            name: name.into(),
            workdir: workdir.into(),
            repo_type: RepoType::Remote { url },
        }
    }

    pub fn url(&self) -> Option<&Url> {
        match &self.repo_type {
            RepoType::Local => None,
            RepoType::Remote { url } => Some(url),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    pub async fn init(path: &Path) -> Result<Self, io::Error> {
        let repo = Self {
            name: String::new(),
            workdir: path.to_path_buf(),
            repo_type: RepoType::Local,
        };

        fs::create_dir_all(&*VOX_DIR).await?;
        fs::create_dir_all(&*OBJ_DIR).await?;
        fs::create_dir_all(&*REFS_DIR).await?;
        fs::write(&*HEAD_DIR, "ref: refs/heads/main\n").await?;

        Ok(repo)
    }

    pub async fn is_initialized(path: &Path) -> Result<bool, io::Error> {
        let vox_dir = path.join(".vox");
        Ok(vox_dir.exists())
    }
}

fn serialize_url<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(url.as_str())
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Url::parse(&s).map_err(serde::de::Error::custom)
}
