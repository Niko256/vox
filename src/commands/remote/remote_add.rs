use crate::commands::{
    config::{
        commands::get_local_config,
        config::{Config, PersistentConfig},
    },
    remote::commands::is_valid_url,
};
use anyhow::Result;
use colored::Colorize;

pub fn add_remote(name: &str, url: &str) -> Result<()> {
    if !is_valid_url(&url) {
        return Err(anyhow::anyhow!("Invalid URL {}", url));
    }

    let config_path = get_local_config()?;
    let mut config = Config::read_from_file(&config_path)?;

    // Check if remote is the same and already exists
    if config
        .remotes()
        .iter()
        .any(|remote| remote.name == name || remote.url == url)
    {
        return Err(anyhow::anyhow!(
            "Remote with name '{}' or URL '{}' already exists",
            name,
            url
        ));
    }

    config.add_remote(name.to_string(), url.to_string());

    config.write_to_file(&config_path)?;

    println!(
        "{}",
        format!("Remote '{}' added successfully", name).green()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remote() {
        let mut config = Config::default();
        config
            .add_remote(
                "origin".to_string(),
                "https://github.com/user/repo.git".to_string(),
            )
            .unwrap();

        assert!(config
            .add_remote(
                "origin".to_string(),
                "https://github.com/other/repo.git".to_string()
            )
            .is_err());

        assert!(config
            .add_remote(
                "upstream".to_string(),
                "https://github.com/user/repo.git".to_string()
            )
            .is_err());

        assert!(config
            .add_remote(
                "upstream".to_string(),
                "https://github.com/other/repo.git".to_string()
            )
            .is_ok());
    }
}
