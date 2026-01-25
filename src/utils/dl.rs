// utils/dl.rs
//! Utilities related to downloading

use std::{
    fs::{
        self,
        File,
    },
    io::{
        self,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
    process::exit,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    time::{
        Duration,
        SystemTime,
    },
};

use fshelpers::mkdir_p;
use futures::{
    StreamExt,
    future::join_all,
};
use httpdate::parse_http_date;
use permitit::Permit;
use reqwest::{
    Client,
    header::{
        // ACCEPT_ENCODING,
        HeaderMap,
        LAST_MODIFIED,
        USER_AGENT,
    },
    redirect::Policy,
};
use thiserror::Error;
use tokio::task;

use crate::unravel;

// TODO: Documentation
// NOTE: Beware the distinction between timeout and connect_timeout
//
/// # Creates a reqwest client
///
/// This client follows up to 16 redirects and has a timeout of 32 seconds. It also sets the user
/// agent to lfstage/3.0.0.
#[allow(clippy::expect_used)]
fn create_client() -> Result<Client, reqwest::Error> {
    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    Client::builder()
        .redirect(Policy::limited(16))
        .http1_ignore_invalid_headers_in_responses(true)
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(
                USER_AGENT,
                user_agent.parse().expect("User agent is invalid"),
            );
            headers
        })
        .connect_timeout(Duration::from_secs(32))
        .build()
}

/// # Parses a 'dl'
///
/// A 'dl' can either be a url, or a url pointing to a filename.
///
/// # Arguments
/// * `dl`          - The raw download to be parsed.
///
/// # Returns
/// * `url`         - The first element in the tuple.
/// * `filename`    - The second element in the tuple.
///
/// # Errors
/// Panics if:
/// - The download does not contain a '/'.
///
/// # Examples
/// - <https://github.com/lloyd/yajl/commit/6fe59ca50dfd65bdb3d1c87a27245b2dd1a072f9.patch> -> yajl-2.1.0-cmake-4-compat.patch
/// - <https://ftp.gnu.org/gnu/bash/bash-5.2.37.tar.gz>
#[allow(clippy::needless_pass_by_value)] // required by multithread shenanigans
// TODO: Find a workaround for ^
pub fn parse_dl<S: Into<String>>(dl: S) -> (String, String) {
    let dl = dl.into();
    // I fucking wish I could use &str -> (&str, &str) here. The function is practically begging
    // but it has to be thread safe :sad:
    if let Some((url, f)) = dl.split_once(" -> ") {
        (url.to_string(), f.to_string())
    } else {
        let (_, f) = dl.rsplit_once('/').unwrap_or_else(|| {
            error!("Invalid url: {dl}");
            exit(1)
        });

        (dl.to_string(), f.to_string())
    }
}

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Extant file: {0}")]
    Extant(PathBuf),

    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

fn get_upstream_modtime(headers: &HeaderMap) -> Option<SystemTime> {
    let h = headers.get(LAST_MODIFIED)?;
    let s = h.to_str().ok()?;
    let t = parse_http_date(s).ok()?;
    Some(t)
}

fn get_local_modtime(path: &Path) -> Option<SystemTime> {
    let m = path.metadata().ok()?;
    let t = m.modified().ok()?;
    Some(t)
}

async fn download_file<P: AsRef<Path>>(
    client: Client,
    url: &str,
    file_path: P,
    download_extant: bool,
) -> Result<(), DownloadError> {
    let file_path = file_path.as_ref();

    // Fetch the url
    debug!("Fetching '{url}'");
    let resp = client
        .get(url)
        // .header(ACCEPT_ENCODING, "identity")
        .send()
        .await?
        .error_for_status()?;

    // Skip extant files, but only if upstream's modtime is greater than local
    if file_path.exists() && !download_extant {
        let upstream_modtime = get_upstream_modtime(resp.headers()).unwrap_or_else(SystemTime::now);
        let local_modtime = get_local_modtime(file_path).unwrap_or(SystemTime::UNIX_EPOCH);

        if upstream_modtime < local_modtime {
            debug!(
                "Skipping download for extant file '{}'",
                file_path.display()
            );
        }
        return Err(DownloadError::Extant(file_path.to_owned()));
    }

    info!("Downloading '{url}'");
    // Create a part file
    let partfile_str = format!("{}.part", file_path.display());
    let mut partfile = File::create(&partfile_str)?;
    let mut stream = resp.bytes_stream();

    // Write the file
    while let Some(chunk) = stream.next().await {
        let data = match chunk {
            | Ok(d) => d,
            | Err(ref e) => {
                error!("Invalid chunk: {e}");
                unravel!(e);
                exit(1)
            },
        };

        partfile.write_all(&data)?;
    }
    partfile.flush()?; // paranoia

    // Move the part file to its final destination
    fs::rename(partfile_str, file_path)?;
    info!("Downloaded '{url}'");
    debug!("Downloaded {}", file_path.display());

    Ok(())
}

pub async fn download_sources<P: AsRef<Path>, Q: AsRef<Path>>(
    sources_list: P,
    sources_dir: Q,
    download_extant: bool,
) -> Result<(), DownloadError> {
    mkdir_p(&sources_dir)?;

    let failed = Arc::new(AtomicBool::new(false));
    let client = match create_client() {
        | Ok(c) => c,
        | Err(ref e) => {
            error!("Failed to create reqwest client: {e}");
            error!("Unable to download sources :(");
            unravel!(e);
            exit(1)
        },
    };

    let dls = read_dls_from_file(sources_list)?;
    trace!("Here's what dls looks like:\n {dls:#?}");
    let mut tasks = Vec::new();

    for dl in dls {
        let client = client.clone();
        let failed = Arc::clone(&failed);
        let (url, filename) = parse_dl(dl);
        let file_path = sources_dir.as_ref().join(&filename);

        let task = task::spawn(async move {
            if let Err(e) = download_file(client, &url, file_path, download_extant)
                .await
                .permit(|e| matches!(e, DownloadError::Extant(_)))
            {
                error!("Failed to download {url} to {filename}: {e}");
                unravel!(e);
                failed.store(false, Ordering::Relaxed);
            }
        });
        tasks.push(task);
    }

    join_all(tasks).await;
    if failed.load(Ordering::Relaxed) {
        error!("Failed to download one or more sources");
        exit(1)
    } else {
        Ok(())
    }
}

/// # Read dl's from a file
///
/// Will fail if the path does not exist, could not be read, contains invalid UTF-8, among other
/// errors (basically anywhere `read_to_string()` would fail).
pub fn read_dls_from_file<P>(path: P) -> Result<Vec<String>, DownloadError>
where
    P: AsRef<Path>,
{
    Ok(fs::read_to_string(path)?
        .lines()
        .filter(|l| !is_comment(l))
        .map(|l| strip_comment_part(l).to_string())
        .collect::<Vec<_>>())
}

/// # Check if a line is a comment or empty
///
/// A line is a comment if it starts with '# ', '; ', or '// ' (leading white space is covered).
#[rustfmt::skip]
fn is_comment(line: &str) -> bool {
    let l = line.trim_start();
    l.is_empty()
        || l.starts_with("# ")
        || l.starts_with("; ")
        || l.starts_with("// ")
}

/// # Strips the comment part from a line
///
/// A comment part is the right side of a line containing ' #', ' //', or ' ;'.
#[rustfmt::skip]
fn strip_comment_part(line: &str) -> &str {
    let comment_starts = [
        line.find(" #"),
        line.find(" //"),
        line.find(" ;"),
    ];

    comment_starts.into_iter().flatten().min().map_or(line, |i| line[..i].trim_end())
}
