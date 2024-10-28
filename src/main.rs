mod cli;
mod commands;
mod utils;
mod objects;
use cli::Cli;
use clap::Parser;


use commands::{add::add_command, cat_file::cat_file_command, hash_object::{
    hash_object_command,
    HashObjectArgs}, init::init_command, tree::write_tree_command
};


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

        cli::Commands::Add { all, file } => {
            if let Err(e) = add_command(all, file) {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        },
        
        cli::Commands::WriteTree => {
            if let Err(e) = write_tree_command() {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }
    }    
}

