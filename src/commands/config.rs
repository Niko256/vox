use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    user: UserConfig,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct UserConfig {
    username: String,
    email: String,
}

impl Config {
    fn read_from_file(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&data)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        let data =
            toml::to_string(self).with_context(|| format!("Failed to serialize config to TOML"))?;
        fs::write(path, data)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }
}

pub fn get_global_config() -> Result<PathBuf> {
    let home_dir = std::env::var("HOME").context("Couldn't find $HOME directory")?;
    Ok(PathBuf::from(home_dir).join(".vcsconfig"))
}

pub fn get_local_config() -> Result<PathBuf> {
    Ok(PathBuf::from(".vcsconfig"))
}

pub fn config_command(global: bool, username: Option<String>, email: Option<String>) -> Result<()> {
    let config_path = if global {
        // Get the global configuration file path
        get_global_config()?
    } else {
        // Get the local configuration file path
        get_local_config()?
    };

    // Load existing configuration or create a new default configuration if the file doesn't exist
    let mut config = if config_path.exists() {
        Config::read_from_file(&config_path)?
    } else {
        Config::default()
    };

    match (username, email) {
        (Some(username), Some(email)) => {
            config.user.username = username;
            config.user.email = email;
            println!("{}", "Updated both username and user's email.".green());
        }

        (Some(username), None) => {
            config.user.username = username;
            println!("{}", "Updated username.".green());
        }

        (None, Some(email)) => {
            config.user.email = email;
            println!("{}", "Updated email.".green());
        }

        (None, None) => {
            println!("{}: {}", "Username".green(), config.user.username);
            println!("{}: {}", "Email".green(), config.user.email);
        }
    }
    config.write_to_file(&config_path)?;
    Ok(())
}
