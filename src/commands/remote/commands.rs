use clap::Subcommand;

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
