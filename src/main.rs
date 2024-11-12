mod cli;
mod command_handler;
mod commands;
pub mod objects;
mod utils;

use clap::Parser;
use cli::Cli;

use command_handler::handle_command;

fn main() {
    let args = Cli::parse();

    if let Err(e) = handle_command(args.command) {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
