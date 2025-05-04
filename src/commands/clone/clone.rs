use crate::commands::config::config::Config;
use crate::storage::objects::ObjectStore;
use crate::storage::objects::pack::Packfile;
use crate::storage::refs::{read_ref, write_ref};
use crate::storage::repo::Repository;
use crate::storage::utils::REFS_DIR;
use anyhow::Context;
use std::path::PathBuf;
use tokio::io;
use url::Url;

/// [Git Server]
///     |
///     | (Git Protocol)
///     v
/// [Transport Layer] -> [Protocol Parser] -> [Object Receiver] -> [Storage System]
///      ^                                          |
///      |                                          v
/// [Clone Command] <------------------------ [Checkout]
///

pub(crate) struct CloneCommand {
    url: Url,
    target: PathBuf,
}

pub struct CloneOptions {}

impl CloneCommand {
    pub async fn execute(&self) -> Result<(), io::Error> {
        !unimplemented!()
    }

    async fn process_deltas(
        &self,
        packfile: Packfile,
        object_storage: &impl ObjectStore,
    ) -> Result<(), io::Error> {
        !unimplemented!()
    }
}

pub async fn clone_command(
    url: impl Into<String>,
    dir: impl Into<Option<PathBuf>>,
) -> Result<(), io::Error> {
    !unimplemented!()
}
