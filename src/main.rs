mod cli;
mod commands;
mod utils;

use cli::Cli;
use commands::{cat_file::cat_file_command, hash_object::{hash_object_command, HashObjectArgs}, init::init_command};
use clap::Parser;

fn main() {
    let args = Cli::parse();

    match args.command {
        cli::Commands::Init => {
            if let Err(e) = init_command() {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        },

        cli::Commands::CatFile { pretty_print, object_hash, show_type, show_size } => {
            if let Err(e) = cat_file_command(pretty_print, object_hash, show_type, show_size) {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        },

        cli::Commands::HashObject { file_path } => {
            if let Err(e) = hash_object_command(HashObjectArgs { file_path }) {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        },

        cli::Commands::Add { all, files } => {
            if all {
                if let Err(e) = add_all_command() {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            } else {
                if let Err(e) = add_command() {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
        },
    }    
}

