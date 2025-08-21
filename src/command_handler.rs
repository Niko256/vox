use std::path::PathBuf;

use crate::cli::Commands;
use crate::commands::branch::branch::branch_command;
use crate::commands::branch::checkout::checkout_command;
use crate::commands::log::log::log_command;
use crate::commands::show::show::show_command;
use crate::commands::write_tree::write_tree::write_tree_command;
use crate::commands::{
    add::add::add_command,
    cat_file::cat_file::cat_file_command,
    commit::commit::commit_command,
    config::commands::config_command,
    diff::diff::diff_command,
    hash_object::hash_object::{HashObjectArgs, hash_object_command},
    index::{ls_files::ls_files_command, rm_index::rm_command},
    init::init::init_command,
    remote::commands::remote_command,
    status::status::status_command,
};
use anyhow::Result;

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Init => {
            init_command().await?;
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
            show_type,
            show_size,
        } => {
            cat_file_command(pretty_print, object_hash, show_type, show_size)?;
        }
        Commands::HashObject { file_path } => {
            hash_object_command(HashObjectArgs { file_path })?;
        }
        Commands::Status => {
            status_command()?;
        }
        Commands::LsFiles { stage } => {
            ls_files_command(stage)?;
        }
        Commands::Rm {
            cashed,
            forced,
            paths,
        } => {
            rm_command(&paths, cashed, forced)?;
        }
        Commands::Add { paths } => {
            add_command(&paths)?;
        }
        Commands::WriteTree { path } => {
            write_tree_command(&path)?;
        }
        Commands::Commit { message, author } => {
            commit_command(&message, author)?;
        }
        Commands::Log { count } => {
            log_command(count)?;
        }
        Commands::Show { commit } => {
            show_command(&commit)?;
        }
        Commands::Branch { name, delete, list } => {
            branch_command(name, delete, list)?;
        }
        Commands::Checkout { target, force } => {
            checkout_command(&target, force, None)?;
        }
        Commands::Config { global, config_cmd } => {
            config_command(global, &config_cmd)?;
        }
        Commands::Remote { remote_cmd } => {
            remote_command(&remote_cmd)?;
        }
        Commands::Diff { from, to } => {
            diff_command(from, to)?;
        }
    }
    Ok(())
}
