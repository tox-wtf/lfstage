// utils/init.rs
//! Initialization utilities

use std::{
    collections::VecDeque,
    fs::{
        self,
        File,
    },
    io::{
        self,
        BufRead,
        BufReader,
        BufWriter,
        ErrorKind,
        Write,
    },
    path::Path,
    process::exit,
    str::FromStr,
    sync::OnceLock,
    time::Instant,
};

use size::{
    Size,
    SizeFormatter,
};

use tempfile::NamedTempFile;
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

use crate::{
    config::{
        CONFIG,
        Config,
    },
};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init() {
    check_perms();

    let log_file = "/var/log/lfstage/lfstage.log";
    log(log_file);
    wrap_trim_log(log_file);
}

fn wrap_trim_log(log_file: &str) {
    let max_size = get_max_log_size();
    match trim_log(log_file, max_size) {
        | Ok(b) => {
            let szf = SizeFormatter::new()
                .with_base(size::Base::Base10)
                .with_style(size::Style::Abbreviated);
            debug!(
                "Trimmed {trimmed} from {log_file} to keep it under {max}",
                trimmed = szf.format(b),
                max = szf.format(max_size),
            );
        },
        | Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                error!("Failed to trim log {e}");
                warn!("You might want to check {log_file} yourself");
            }
        },
    }
}

#[allow(clippy::cast_sign_loss)]
fn get_max_log_size() -> u64 {
    if let Ok(sz) = Size::from_str(&CONFIG.log_max_size) {
        return sz.bytes() as u64
    }

    warn!("Failed to parse log_max_size from config");
    warn!("Falling back to default");
    if let Ok(sz) = Size::from_str(&Config::default().log_max_size) {
        return sz.bytes() as u64
    }

    warn!("I fucked up the default config. Please report this!");
    warn!("Continuing with yet another fallback");
    10 * 1024 * 1024 // 10 MB
}

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
    let (dir, file) = path
        .as_ref()
        .rsplit_once('/')
        .expect("Log file path should contain /");
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

/// # Trims a log file until it's under a maximum size
///
/// Trimming means deleting lines from the top of the file
///
/// # Arguments
/// * `path`        - The path to the log file to be trimmed
/// * `max_size`    - The maximum size of the log file, in bytes
///
/// # Returns
/// Bytes trimmed
///
/// # Errors
/// - Log file does not exist (`NotFound` should be handled when called)
/// - Other I/O errors or something
///
/// # Examples
/// ```rust
/// const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
/// trim_log("hello.log", MAX_SIZE).permit(|e| e.kind() == std::io::ErrorKind::NotFound)
/// ```
fn trim_log<P: AsRef<Path>>(path: P, max_size: u64) -> io::Result<u64> {
    let path = path.as_ref();
    let size = fs::metadata(path)?.len();

    if size <= max_size {
        return Ok(0);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = VecDeque::new();
    let mut total_size = 0;

    for line in reader.lines() {
        let line = line?;
        let line_size = (line.len() + 1) as u64; // account for \n

        total_size += line_size;
        lines.push_back((line, line_size));

        while total_size > max_size {
            if let Some((_, removed_size)) = lines.pop_front() {
                total_size -= removed_size;
            }
        }
    }

    let mut temp_file = NamedTempFile::new()?;
    {
        let mut writer = BufWriter::new(&mut temp_file);
        for (line, _) in &lines {
            writeln!(writer, "{line}")?;
        }
    }

    temp_file.persist(path)?;
    Ok(size - total_size)
}

#[cfg(test)]
mod test {
    use std::{
        fs,
        io::Write,
    };

    use tempfile::NamedTempFile;

    use crate::utils::init::trim_log;

    #[test]
    #[allow(clippy::expect_used)]
    #[allow(clippy::unwrap_used)]
    fn trim_log_file() {
        // Setup
        const MAX_SIZE: u64 = 1000;
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

        // Write some ~~junk~~ data to exceed MAX_SIZE
        writeln!(temp_file, "Reasons why Arch is the best:").unwrap();
        for i in 0..256 {
            writeln!(temp_file, "{i}. Arch is the best!").unwrap();
        }
        writeln!(temp_file, "The evidence is truly irrefutable.").unwrap();
        writeln!(
            temp_file,
            "Further reading: https://wiki.archlinux.org/title/Arch_is_the_best"
        )
        .unwrap();

        // More setup
        let path = temp_file.path();
        let before_size = fs::metadata(path).unwrap().len();

        // Trim the log
        let trimmed = trim_log(path, MAX_SIZE).expect("Failed to trim log");

        // Ensure size is no greater than `MAX_SIZE`
        let after_size = fs::metadata(path).unwrap().len();
        assert!(after_size <= MAX_SIZE, "trim_log() doesn't work :(");

        // Ensure stuff was actually trimmed
        assert!(trimmed > 0);
        assert!(after_size < before_size);

        // Ensure the accuracy of `trimmed`
        assert_eq!(trimmed, before_size - after_size);

        // Ensure newlines are present
        let contents = fs::read_to_string(path).unwrap();
        let lines = contents.lines().collect::<Vec<_>>();

        // Ensure end is intact
        assert!(lines.last().unwrap().contains("Further reading: "));
    }
}
