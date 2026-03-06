pub mod build;
pub mod clean;
pub mod download;
pub mod export;
pub mod import;
pub mod list;

use std::{
    io,
    path::PathBuf,
};

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{
    Parser,
    Subcommand,
};
use thiserror::Error;

use crate::utils::dl::DownloadError;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Cyan.on_default().bold())
    .usage(AnsiColor::Cyan.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::White.on_default());

#[derive(Parser)]
#[command(
    name = "LFStage",
    version = env!("CARGO_PKG_VERSION"),
    styles = STYLES,
    author,
    about,
    infer_subcommands = true,
    infer_long_args = true,
    disable_help_subcommand = true,
    propagate_version = true,
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[non_exhaustive]
enum Commands {
    Build(build::Cmd),
    Clean(clean::Cmd),
    List(list::Cmd),
    Import(import::Cmd),
    Export(export::Cmd),
    Download(download::Cmd),
}

#[rustfmt::skip]
#[derive(Debug, Error)]
pub enum CmdError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    // #[error("Invalid argument: {0}")]
    // InvalidArgument(String),

    #[error("Missing component: {0}")]
    MissingComponent(PathBuf),

    #[error("Download error: {0}")]
    Download(#[from] DownloadError),

    // #[error("Script failed: {0}")]
    // Command(String),
}

impl Cli {
    pub async fn run(&self) -> Result<(), CmdError> {
        match &self.command {
            | Commands::Build(cmd) => cmd.run().await,
            | Commands::Clean(cmd) => cmd.run(),
            | Commands::List(cmd) => cmd.run(),
            | Commands::Import(cmd) => cmd.run(),
            | Commands::Export(cmd) => cmd.run(),
            | Commands::Download(cmd) => cmd.run().await,
        }
    }
}
