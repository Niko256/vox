mod cli;
mod commands;
pub mod objects;
mod utils;
use clap::Parser;
use cli::Cli;

use commands::{
    cat_file::cat_file_command,
    hash_object::{hash_object_command, HashObjectArgs},
    init::init_command,
};

fn main() {
    let args = Cli::parse();

    match args.command {
        cli::Commands::Init => {
            if let Err(e) = init_command() {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }

        cli::Commands::CatFile {
            pretty_print,
            object_hash,
            show_type,
            show_size,
        } => {
            if let Err(e) = cat_file_command(pretty_print, object_hash, show_type, show_size) {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }

        cli::Commands::HashObject { file_path } => {
            if let Err(e) = hash_object_command(HashObjectArgs { file_path }) {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
