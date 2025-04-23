mod cli;
mod command_handler;
mod commands;
mod connection;
pub mod storage;

use clap::Parser;
use cli::Cli;

use command_handler::handle_command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    handle_command(cli.command).await?;
    Ok(())
}
