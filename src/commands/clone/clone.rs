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
    fn validate_url(&self) -> Result<()>;
    fn ensure_target_dir(&self) -> Result<()>;
    fn configure_repository(&self, repo: &mut Repository) -> Result<()>;
    fn setup_remotes(&self, repo: &mut Repository) -> Result<()>;
}

mod errors {
    use core::error;

    #[derive(Error, Debug)]
    pub enum CloneErrors {
        #[error("Failed to initialize repository")]
        InitErr,

        #[error("Failed to fetch data from remore repository")]
        FetchErr,

        #[error("Failed to checkout")]
        CheckoutErr,
    }
}

pub async fn clone_command(url: String, dir: Option<PathBuf>) -> Result<String> {}
