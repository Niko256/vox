use crate::commands::config::config::Config;
use crate::commands::remote::commands::RemoteRepository;
use crate::storage::refs::{read_ref, write_ref};
use crate::storage::utils::REFS_DIR;
use std::path::PathBuf;
use url::Url;

/// TODO:
///
/// 1) Prepare clone:
///     - vox init
///     - parse URL
///
/// 2) Fetch phase
///     - handle connection
///     - refspecs
///     - 'negotiation'
///
/// 3) data recieving
///     - 'packfile' deserialization
///     - checkout
///

pub(crate) struct CloneCommand {
    url: Url,
    target: PathBuf,
    options: Option<CloneOptions>,
}

pub struct CloneOptions {}

impl CloneCommand {
    fn validate_url(&self) -> Result<()> {
        !unimplemented!()
    }
    fn ensure_target_dir(&self) -> Result<()> {
        !unimplemented!()
    }
    fn configure_repository(&self, repo: &mut Repository) -> Result<()> {
        !unimplemented!()
    }
    fn setup_remotes(&self, repo: &mut Repository) -> Result<()> {
        !unimplemented!()
    }
}

pub async fn clone_command(url: String, dir: Option<PathBuf>) -> Result<String> {
    !unimplemented!()
}
