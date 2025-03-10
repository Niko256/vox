use crate::commands::config::{
    commands::get_local_config,
    config::{Config, PersistentConfig},
};
use anyhow::Result;
use colored::Colorize;

pub fn remove_remote(name: &str) -> Result<()> {
    let config_path = get_local_config()?;
    let mut config = Config::read_from_file(&config_path)?;

    config.remove_remote(name)?;

    config.write_to_file(&config_path)?;

    println!("{}", format!("Remote '{}' removed", name).green());
    Ok(())
}
