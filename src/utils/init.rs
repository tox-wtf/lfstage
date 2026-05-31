// utils/init.rs
//! Initialization utilities

use std::io;
use std::process::exit;
use std::str::FromStr;
use std::time::Instant;

use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::FormatTime;

use crate::config::CONFIG;

pub fn init() {
    check_perms();

    log();
}

#[inline]
fn check_perms() {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("Run this as root");
        exit(1);
    }
}

/// # Uptime struct for timestamp formatting in logs
struct Uptime(Instant);

impl Uptime {
    /// # Create a new [`Uptime`]
    #[inline]
    #[must_use]
    fn new() -> Self { Self(Instant::now()) }
}

impl FormatTime for Uptime {
    #[inline]
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let elapsed = self.0.elapsed();
        let s = elapsed.as_secs();
        let ms = elapsed.subsec_millis();
        write!(w, "{s:>4}.{ms:03}")
    }
}

#[allow(clippy::expect_used, clippy::unwrap_used)]
fn log() {
    let debug = cfg!(debug_assertions);
    let level = LevelFilter::from_str(&CONFIG.log_level).unwrap_or(match debug {
        | true => LevelFilter::TRACE,
        | false => LevelFilter::DEBUG,
    });

    let filter = EnvFilter::new(format!("{level},rustls=warn,hyper_util=warn,reqwest=warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(debug)
        .with_line_number(debug)
        .with_timer(Uptime::new())
        .with_writer(io::stderr)
        .with_ansi_sanitization(false)
        .compact()
        .init();
}
