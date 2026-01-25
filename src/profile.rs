// profile.rs
//! The profile struct and related code

use std::{
    ffi::OsStr,
    fmt,
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::exit,
};

use fshelpers::mkdir_p;
use is_executable::IsExecutable;

use crate::{
    exec,
    utils::dl::{
        DownloadError,
        download_sources,
        parse_dl,
        read_dls_from_file,
    },
};

#[derive(Debug)]
pub struct Profile {
    pub name: String,
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.name) }
}

impl Profile {
    pub fn new<S: Into<String>>(name: S) -> Self { Self { name: name.into() } }

    pub fn tmpdir(&self) -> PathBuf { Path::new("/tmp/lfstage").join(&self.name) }

    pub fn stagefilenamefile(&self) -> PathBuf { self.tmpdir().join("stagefilename") }

    pub fn timestampfile(&self) -> PathBuf { self.tmpdir().join("timestamp") }

    pub fn scriptdir(&self) -> PathBuf {
        Path::new("/var/lib/lfstage/profiles")
            .join(&self.name)
            .join("scripts")
    }

    pub fn stagedir(&self) -> PathBuf {
        Path::new("/var/cache/lfstage/profiles")
            .join(&self.name)
            .join("stages")
    }

    pub fn sourcesdir(&self) -> PathBuf {
        Path::new("/var/cache/lfstage/profiles")
            .join(&self.name)
            .join("sources")
    }

    pub fn sourceslist(&self) -> PathBuf {
        Path::new("/var/lib/lfstage/profiles")
            .join(&self.name)
            .join("sources")
    }

    pub fn get_registered_sources(&self) -> Vec<String> {
        read_dls_from_file(self.sourceslist())
            .unwrap_or_else(|e| {
                error!("Failed to read dls from sources list: {e}");
                exit(1)
            })
            .iter()
            .map(|dl| parse_dl(dl).1)
            .collect::<Vec<_>>()
    }

    pub fn collect_build_scripts(&self) -> Vec<PathBuf> {
        // Gather all profile-specific scripts
        let mut scripts = self
            .scriptdir()
            .read_dir()
            .unwrap_or_else(|e| {
                warn!("Failed to read scripts directory for profile '{self}': {e}");
                exit(1)
            })
            .filter_map(|e| match e {
                | Ok(e) => Some(e),
                | Err(e) => {
                    warn!("Entry could not be accessed: {e}");
                    warn!("Ignoring it");
                    None
                },
            })
            .map(|e| e.path())
            .filter(|p| !p.is_dir() && p.is_executable())
            .filter_map(|p| {
                let str = p
                    .file_name()
                    .unwrap_or_else(|| {
                        warn!("Path {p:?} has no file name!");
                        warn!("Ignoring it");
                        OsStr::new("") // kinda hacky but wtv
                    })
                    .to_string_lossy();
                if str.chars().take(2).all(|c| c.is_ascii_digit()) { Some(p) } else { None }
            })
            .collect::<Vec<_>>();

        // Sort them
        scripts.sort_by_key(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.split_once('-'))
                .and_then(|(prefix, _)| prefix.parse::<u32>().ok())
        });

        scripts
    }

    pub fn run_build_scripts(&self) {
        for script in self.collect_build_scripts() {
            info!("Running build script {}", script.display());
            if let Err(e) = exec!(&self; &script) {
                error!("Failure in {}: {e}", script.display());
                exit(1)
            }
        }
    }

    pub fn setup_sources(&self) -> std::io::Result<()> {
        let registered = self.get_registered_sources();

        let sources = self
            .sourcesdir()
            .read_dir()?
            .filter_map(|e| match e {
                | Ok(e) => Some(e),
                | Err(e) => {
                    warn!("Entry could not be accessed: {e}");
                    warn!("Ignoring it");
                    None
                },
            })
            .map(|e| e.path())
            .filter(|p| {
                registered.contains(
                    &p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                )
            })
            .collect::<Vec<_>>();

        debug!("Found registered sources: {sources:#?}");

        let lfs_sources = Path::new("/var/lib/lfstage/mount/sources");
        mkdir_p(lfs_sources)?;

        for source in sources {
            let dest = lfs_sources.join(source.components().next_back().unwrap());
            fs::copy(source, dest)?;
        }

        Ok(())
    }

    pub fn save_stagefile(&self) -> std::io::Result<()> {
        mkdir_p(self.stagedir())?;
        if exec!(&self; "/usr/lib/lfstage/scripts/save.sh").is_err() {
            error!("Failed to save stage file");
            exit(1)
        }

        info!(
            "Saved stage file to {}",
            fs::read_to_string(self.stagefilenamefile())?
        );

        Ok(())
    }

    pub async fn download_sources(&self, download_extant: bool) -> Result<(), DownloadError> {
        download_sources(self.sourceslist(), self.sourcesdir(), download_extant).await
    }
}
