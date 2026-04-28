use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "krate",
    version,
    about = "Fast TUI downloader for Minecraft server plugins"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search and download one or more plugins.
    Get {
        /// Plugin search text.
        query: String,
    },
    /// Choose the plugin provider server.
    Server,
    /// Update installed plugins in the current directory.
    Update {
        /// Target Minecraft version. If omitted, Krate asks in the TUI.
        #[arg(short, long)]
        version: Option<String>,

        /// Target server software/loader. If omitted, Krate asks in the TUI.
        #[arg(short, long)]
        loader: Option<String>,
    },
}
