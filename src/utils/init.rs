// utils/init.rs
//! Initialization utilities

use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Instant;
use std::{fs, io};

use tracing::metadata::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::config::CONFIG;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
const LOG_FILE: &str = "/var/log/lfstage/lfstage.log";

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
    if let Some(parent) = Path::new(LOG_FILE).parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create directory");
    }
    fs::write(LOG_FILE, "").expect("Failed to truncate log file");

    let debug = cfg!(debug_assertions);
    let level = LevelFilter::from_str(&CONFIG.log_level).unwrap_or(match debug {
        | true => LevelFilter::TRACE,
        | false => LevelFilter::DEBUG,
    });

    let filter = EnvFilter::new(format!("{level},hyper_util=warn,reqwest=warn"));

    let (dir, file) = LOG_FILE.rsplit_once('/').unwrap_or((".", LOG_FILE));
    let file_appender = rolling::never(dir, file);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(debug)
        .with_line_number(debug)
        .with_timer(Uptime::new())
        .with_writer(file_writer.and(io::stdout))
        .compact()
        .init();

    LOG_GUARD.set(guard).expect("logs were inited more than once");
}
