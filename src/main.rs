mod cli;
mod command_handler;
mod commands;
pub mod objects;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{
    cat_file::cat_file_command,
    hash_object::{hash_object_command, HashObjectArgs},
    index::ls_files,
    index::rm_index,
    init::init_command,
    status,
};

use command_handler::handle_command;

fn main() {
    let args = Cli::parse();

    if let Err(e) = handle_command(args.command) {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
