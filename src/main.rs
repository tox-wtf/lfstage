// src/main.rs

mod cli;
mod config;
mod profile;
mod utils;

use std::process::exit;

use clap::Parser;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    utils::init::init();
    if let Err(e) = cli::Cli::parse().run().await {
        error!("{e}");
        unravel!(e);
        exit(1);
    }
}
