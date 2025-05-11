use crate::connection::VoxTransport;
use crate::storage::objects::pack::Packfile;

use super::clone::CloneCommand;
use anyhow::Result;
use std::collections::HashMap;

impl CloneCommand {
    pub async fn fetch_refs(&self, transport: &VoxTransport) -> Result<HashMap<String, String>> {
        let server_refs = transport.list_refs().await?;
        let mut refs = HashMap::new();

        for r in server_refs {
            refs.insert(r.name, r.hash);
        }

        Ok(refs)
    }

    pub async fn fetch_packfile(
        &self,
        transport: &VoxTransport,
        refs: &HashMap<String, String>,
    ) -> Result<Packfile> {
        let want: Vec<String> = refs.values().cloned().collect();
        let pack_data = transport.fetch_packfile(&want).await?;
        Packfile::deserialize(&pack_data)
    }
}
