// utils/dl.rs
//! Utilities related to downloading

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use std::{fmt, string};

use futures::StreamExt;
use futures::future::join_all;
use permitit::Permit;
use reqwest::Client;
use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::redirect::Policy;
use thiserror::Error;
use tokio::task;

use crate::profile::Profile;

// TODO: Documentation
// NOTE: Beware the distinction between timeout and connect_timeout
//
/// # Creates a reqwest client
///
/// This client follows up to 32 redirects and has a connection timeout of 120 seconds. It also
/// sets the user agent to crate/version.
#[allow(clippy::expect_used)]
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    Client::builder()
        .redirect(Policy::limited(32))
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, user_agent.parse().expect("User agent is invalid"));
            headers
        })
        .connect_timeout(Duration::from_mins(2))
        .build()
        .expect("Failed to build client")
});

#[derive(Debug)]
pub struct Download {
    pub url:  String,
    pub dest: String,
}

impl fmt::Display for Download {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{} -> {}", self.url, self.dest) }
}

impl FromStr for Download {
    type Err = DownloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((u, f)) = s.split_once(" -> ") {
            return Ok(Self {
                url:  u.to_string(),
                dest: f.to_string(),
            });
        }

        let (_, f) = s.rsplit_once('/').ok_or_else(|| DownloadError::InvalidUrl(s.to_string()))?;
        Ok(Self {
            url:  s.to_string(),
            dest: f.to_string(),
        })
    }
}

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Extant file: {0}")]
    Extant(PathBuf),

    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("UTF-8 Error: {0}")]
    FromUtf8(#[from] string::FromUtf8Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

async fn download_file<P: AsRef<Path>>(url: &str, file_path: P, download_extant: bool) -> Result<(), DownloadError> {
    let file_path = file_path.as_ref();

    // Skip extant files
    if file_path.exists() && !download_extant {
        debug!("Skipping download for extant file '{}'", file_path.display());
        return Err(DownloadError::Extant(file_path.to_owned()));
    }

    // Fetch the url
    let resp = CLIENT.get(url).send().await?.error_for_status()?;

    info!("Downloading '{url}'");
    // Create a part file
    let partfile_str = format!("{}.part", file_path.display());
    let mut partfile = File::create(&partfile_str)?;
    let mut stream = resp.bytes_stream();

    // Write the file
    while let Some(chunk) = stream.next().await {
        let data = chunk.inspect_err(|e| error!("Invalid chunk: {e}"))?;
        partfile.write_all(&data)?;
    }

    partfile.flush()?; // paranoia

    // Move the part file to its final destination
    fs::rename(partfile_str, file_path)?;
    info!("Downloaded '{}'", file_path.display());

    Ok(())
}

impl Profile {
    pub async fn download_sources(&self, download_extant: bool) -> Result<(), DownloadError> {
        let sources_dir = self.sources_dir();
        if !sources_dir.exists() {
            fs::create_dir_all(&sources_dir)?;
        }

        let failed = Arc::new(AtomicBool::new(false));

        let dls = self.read_dls()?;
        trace!("Here's what dls looks like:\n {dls:#?}");
        let mut tasks = Vec::new();

        for dl in dls {
            let failed = Arc::clone(&failed);
            let dest = sources_dir.join(&dl.dest);

            let task = task::spawn(async move {
                if let Err(e) = download_file(&dl.url, &dest, download_extant)
                    .await
                    .permit(|e| matches!(e, DownloadError::Extant(_)))
                {
                    error!("Failed to download {} to {}: {e}", dl.url, dest.display());
                    failed.store(false, Ordering::Relaxed);
                }
            });

            tasks.push(task);
        }

        join_all(tasks).await;
        if failed.load(Ordering::Relaxed) {
            error!("Failed to download one or more sources");
            exit(1)
        }

        Ok(())
    }

    /// # Read dl's from the sources file
    ///
    /// Will fail if the path does not exist, could not be read, contains invalid UTF-8, among other
    /// errors (basically anywhere `read_to_string()` would fail).
    #[inline]
    pub fn read_dls(&self) -> Result<Vec<Download>, DownloadError> {
        String::from_utf8(Command::new(self.sources_file()).env("ENVS", self.envs_dir().as_os_str()).output()?.stdout)?
            .lines()
            .filter(|l| !is_comment(l))
            .map(|l| strip_comment_part(l).to_string())
            .map(|dl| dl.parse())
            .collect::<Result<_, _>>()
    }

    pub fn get_registered_sources(&self) -> Vec<String> {
        self.read_dls()
            .unwrap_or_else(|e| {
                error!("Failed to read dls from sources list: {e}");
                exit(1)
            })
            .iter()
            .map(|dl| dl.dest.clone())
            .collect()
    }
}

/// # Check if a line is a comment or empty
///
/// A line is a comment if it starts with '#' (or is empty after trimming leading whitespace)
#[rustfmt::skip]
#[inline]
fn is_comment(line: &str) -> bool {
    let l = line.trim_start();
    l.is_empty()
        || l.starts_with('#')
}

/// # Strips the comment part from a line
///
/// A comment part is the right side of a line containing '  #'
#[rustfmt::skip]
#[inline]
fn strip_comment_part(line: &str) -> &str {
    line.rsplit_once("  #").map_or(line, |(l, _)| l)
}
