use crate::commands::{config::config::Config, remote::commands::RemoteRepository};

pub struct PrepareFetch {
    repository: RemoteRepository,
    remote_config: Config,
}

impl PrepareFetch {
    fn new(repo: RemoteRepository, config: Config) -> Result<Self, std::io::Error>;
    async fn prepare_fetch(&mut self) -> Result<(), std::io::Error>;
    async fn config_remote(&mut self) -> Result<(), std::io::Result>;
}
