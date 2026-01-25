// src/main.rs

#![deny(clippy::perf, clippy::todo, clippy::complexity)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused
)]

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
