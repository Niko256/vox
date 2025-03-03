use crate::commands::config::config::{Config, ConfigCommands, PersistentConfig};
use anyhow::{Context, Result};
use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
}

pub fn get_global_config() -> Result<PathBuf> {
    let home_dir = std::env::var("HOME").context("Couldn't find $HOME directory")?;
    Ok(PathBuf::from(home_dir).join(".vcsconfig"))
}

pub fn get_local_config() -> Result<PathBuf> {
    let curr_dir = std::env::current_dir().context("Couldn't get current directory")?;
    Ok(curr_dir.join(".vcsconfig"))
}

fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

pub fn config_command(config_command: &ConfigCommands) -> Result<()> {
    let config_path = get_local_config()?;
    let mut config = Config::read_from_file(&config_path)?;

    match config_command {
        ConfigCommands::Show => {
            println!("{}", "Current configuration:".bold().green());
            println!("{}: {}", "Username".green(), config.username());
            println!("{}: {}", "Email".green(), config.email());
            println!("{}: {}", "Server URL".green(), config.url());
            println!(
                "{}: {}",
                "API Key".green(),
                config
                    .api_key()
                    .cloned()
                    .unwrap_or_else(|| "Not set".to_string())
            );
        }
        ConfigCommands::SetUsername { username } => {
            config.set_username(username.trim().to_string());
            println!("{}", "Updated username.".green());
        }
        ConfigCommands::SetEmail { email } => {
            let trimmed_email = email.trim();
            if !is_valid_email(trimmed_email) {
                return Err(anyhow::anyhow!("Invalid email format: {}", email));
            }
            config.set_email(trimmed_email.to_string());
            println!("{}", "Updated email.".green());
        }
        ConfigCommands::SetUrl { url } => {
            config.set_url(url.trim().to_string());
            println!("{}", "Updated server URL.".green());
        }
        ConfigCommands::SetApiKey { api_key } => {
            config.set_api_key(Some(api_key.trim().to_string()));
            println!("{}", "Updated API key.".green());
        }
    }

    config.write_to_file(&config_path)?;
    Ok(())
}
