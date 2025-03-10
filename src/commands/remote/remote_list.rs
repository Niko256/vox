use crate::commands::config::{
    commands::get_local_config,
    config::{Config, PersistentConfig},
};
use anyhow::Result;
use colored::Colorize;

pub fn list_remotes() -> Result<()> {
    let config_path = get_local_config()?;
    let config = Config::read_from_file(&config_path)?;

    if config.remotes().is_empty() {
        println!("{}", format!("No remotes found.").red());
    } else {
        println!("{}", format!("Remotes:").green());
        for remote in config.remotes() {
            println!(
                "{}",
                format!("{}: {}", remote.name.green(), remote.url.blue())
            );
        }
    }

    Ok(())
}
