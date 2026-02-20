// utils/init.rs
//! Initialization utilities

use std::{
    io,
    fs,
    process::exit,
    str::FromStr,
    sync::OnceLock,
    time::Instant,
};

use tracing::metadata::LevelFilter;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling,
};
use tracing_subscriber::{
    EnvFilter,
    fmt::{
        time::FormatTime,
        writer::MakeWriterExt,
    },
};

use crate::config::CONFIG;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init() {
    check_perms();

    let log_file = "/var/log/lfstage/lfstage.log";
    let _ = fs::remove_file(log_file);
    log(log_file);
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
    fn new() -> Self { Self(Instant::now()) }
}

impl FormatTime for Uptime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let elapsed = self.0.elapsed();
        let s = elapsed.as_secs();
        let ms = elapsed.subsec_millis();
        write!(w, "{s:>4}.{ms:03}")
    }
}

#[allow(clippy::expect_used, clippy::unwrap_used)]
fn log<P: AsRef<str>>(path: P) {
    let path = path.as_ref();

    let (dir, file) = path.rsplit_once('/').unwrap_or((".", path));
    let file_appender = rolling::never(dir, file);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let level = LevelFilter::from_str(&CONFIG.log_level).unwrap_or(LevelFilter::DEBUG);
    let filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .with_env_var("LOG_LEVEL")
        .from_env_lossy()
        .add_directive("fshelpers=warn".parse().unwrap())
        .add_directive("hyper_util=warn".parse().unwrap())
        .add_directive("reqwest=warn".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(cfg!(debug_assertions))
        .with_line_number(cfg!(debug_assertions))
        .with_timer(Uptime::new())
        .with_writer(file_writer.and(io::stdout))
        .compact()
        .init();

    LOG_GUARD.set(guard).expect("logs were inited more than once");
}
