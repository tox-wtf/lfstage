// profile.rs
//! The profile struct and related code

use std::{
    ptr,
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
        read_dls_from_file,
    },
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Profile {
    pub name: str,
}

impl AsRef<Self> for Profile {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<str> for Profile {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl AsRef<Profile> for str {
    #[inline]
    fn as_ref(&self) -> &Profile {
        Profile::new(self)
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", &self.name) }
}

impl Profile {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
        // SAFETY: Trust me bro
        unsafe { &*(ptr::from_ref(s.as_ref()) as *const Self) }
    }

    #[inline]
    pub fn tmp_dir(&self) -> PathBuf { Path::new("/tmp/lfstage").join(&self.name) }

    #[inline]
    pub fn stagefilename_file(&self) -> PathBuf { self.tmp_dir().join("stagefilename") }

    #[inline]
    pub fn timestamp_file(&self) -> PathBuf { self.tmp_dir().join("timestamp") }

    #[inline]
    pub fn profile_lib_dir(&self) -> PathBuf {
        Path::new("/var/lib/lfstage/profiles")
            .join(&self.name)
    }

    #[inline]
    pub fn profile_cache_dir(&self) -> PathBuf {
        Path::new("/var/cache/lfstage/profiles")
            .join(&self.name)
    }

    #[inline]
    pub fn envs_dir(&self) -> PathBuf { self.profile_lib_dir().join("envs") }

    #[inline]
    pub fn scripts_dir(&self) -> PathBuf { self.profile_lib_dir().join("scripts") }

    #[inline]
    pub fn stages_dir(&self) -> PathBuf { self.profile_cache_dir().join("stages") }

    #[inline]
    pub fn sources_dir(&self) -> PathBuf { self.profile_cache_dir().join("sources") }

    #[inline]
    pub fn sources_file(&self) -> PathBuf { self.profile_lib_dir().join("sources") }

    pub fn get_registered_sources(&self) -> Vec<String> {
        read_dls_from_file(self.sources_file())
            .unwrap_or_else(|e| {
                error!("Failed to read dls from sources list: {e}");
                exit(1)
            })
            .iter()
            .map(|dl| dl.dest.clone())
            .collect()
    }

    pub fn collect_build_scripts(&self) -> Vec<PathBuf> {
        // Gather all profile-specific scripts
        let mut scripts = self
            .scripts_dir()
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
            .sources_dir()
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
            let Some(source_filename) = source.file_name() else {
                error!("Invalid source: {}", source.display());
                exit(1);
            };

            let dest = lfs_sources.join(source_filename);
            fs::copy(source, dest)?;
        }

        Ok(())
    }

    pub fn save_stagefile(&self) -> std::io::Result<()> {
        mkdir_p(self.stages_dir())?;
        if exec!(&self; "/usr/lib/lfstage/scripts/save.sh").is_err() {
            error!("Failed to save stage file");
            exit(1)
        }

        info!(
            "Saved stage file to {}",
            fs::read_to_string(self.stagefilename_file())?
        );

        Ok(())
    }

    pub async fn download_sources(&self, download_extant: bool) -> Result<(), DownloadError> {
        download_sources(self.sources_file(), self.sources_dir(), download_extant).await
    }
}
