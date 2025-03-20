use super::{
    remote_add::add_remote, remote_list::list_remotes, remote_remove::remove_remote,
    remote_rename::rename_remote, remote_show::show_remote,
};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RemoteRepository {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Subcommand)]
pub enum RemoteCommands {
    #[command(about = "Add a new remote repository")]
    Add { name: String, url: String },

    #[command(about = "Remove a remote repository")]
    Remove { name: String },

    #[command(about = "Rename a remote repository")]
    Rename { old_name: String, new_name: String },

    #[command(about = "Show info about a remote repository")]
    Show { name: String },

    #[command(about = "List all remote repositories")]
    List,
}

pub fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

pub fn remote_command(command: &RemoteCommands) -> Result<()> {
    match command {
        RemoteCommands::Add { name, url } => add_remote(name, url),
        RemoteCommands::Rename { old_name, new_name } => rename_remote(old_name, new_name),
        RemoteCommands::Remove { name } => remove_remote(name),
        RemoteCommands::List => list_remotes(),
        RemoteCommands::Show { name } => show_remote(name),
    }
}
