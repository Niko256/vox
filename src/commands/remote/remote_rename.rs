use crate::commands::config::{
    commands::get_local_config,
    config::{Config, PersistentConfig},
};
use anyhow::Result;
use colored::Colorize;

pub fn rename_remote(old_name: &str, new_name: &str) -> Result<()> {
    let config_path = get_local_config()?;
    let mut config = Config::read_from_file(&config_path)?;

    config.rename_remote(old_name, new_name)?;
    config.write_to_file(&config_path)?;

    println!(
        "{}",
        format!(
            "Remote '{}' renamed to '{}' successfully",
            old_name, new_name
        )
        .green()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_remote() {
        let mut config = Config::default();
        config
            .add_remote(
                "origin".to_string(),
                "https://github.com/user/repo.git".to_string(),
            )
            .unwrap();
        config.remove_remote("origin").unwrap();
        assert!(!config.remotes().iter().any(|r| r.name == "origin"));
    }
}
