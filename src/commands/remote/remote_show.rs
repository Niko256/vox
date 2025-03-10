use crate::commands::config::{
    commands::get_local_config,
    config::{Config, PersistentConfig},
};
use anyhow::Result;
use colored::Colorize;

pub fn show_remote(name: &str) -> Result<()> {
    let config_path = get_local_config()?;
    let config = Config::read_from_file(&config_path)?;

    let remote = config.get_remote(name)?;

    println!("{}", format!("Remote: '{}'", remote.name).green());
    println!("{}", format!("URL: '{}'", remote.url).green());
    Ok(())
}
