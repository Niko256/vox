mod cli;
mod commands;
mod utils;

use cli::Cli;
use commands::{init::init_command, cat_file::cat_file_command, help::help_command};
use clap::Parser;

fn main() {
    let args = Cli::parse();

    match args.command {
        cli::Commands::Init => init_command().unwrap_or_else(|e| {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }),
        cli::Commands::CatFile { pretty_print, object_hash, show_type, show_size } => cat_file_command(pretty_print, object_hash, show_type, show_size).unwrap_or_else(|e| {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }),
        cli::Commands::HelpCommand => help_command().unwrap_or_else(|e| {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }),
    }
}
