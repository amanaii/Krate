mod cli;
mod commands;
mod config;
mod downloader;
mod http;
mod models;
mod providers;
mod tui;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Get { query }) => commands::get::run(&query).await,
        Some(Command::Server) => commands::server::run().await,
        Some(Command::Update { version, loader }) => commands::update::run(version, loader).await,
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}
