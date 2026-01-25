// cli/build.rs

use clap::Args;

use super::CmdError;
use crate::{
    config::CONFIG,
    profile::Profile,
    utils::dl::read_dls_from_file,
};

#[derive(Args, Debug)]
pub struct Cmd {
    #[arg(default_value = CONFIG.default_profile.as_str())]
    pub profile: String,

    /// Whether to forcibly download sources
    #[arg(short, long)]
    pub force: bool,

    /// Whether to perform a dry-run
    #[arg(short, long)]
    pub dry: bool,
}

impl Cmd {
    /// # Runs the download subcommand
    ///
    /// The download subcommand downloads the sources for a stage file profile.
    ///
    /// # Arguments
    /// * `self.profile`    - The profile to target, defaults to "x86_64-glibc-tox-stage2".
    /// * `self.dry`        - If true, perform a dry run, building nothing.
    ///
    /// # Errors
    /// This function returns a `CmdError` if:
    /// - The script directory couldn't be read.
    /// - One of the scripts failed.
    pub async fn run(&self) -> Result<(), CmdError> {
        let profile = Profile::new(&self.profile);

        if !profile.sourceslist().exists() {
            error!("Sources list for profile '{}' does not exist", self.profile);
            return Err(CmdError::MissingComponent(profile.sourceslist()));
        }

        if self.dry {
            let dls = read_dls_from_file(profile.sourceslist())?;
            println!(
                "Would download the following to '{}':",
                profile.sourcesdir().display()
            );

            for dl in &dls {
                println!(" - {dl}");
            }

            return Ok(())
        }

        info!("Downloading sources for '{profile}'");
        profile.download_sources(self.force).await?;
        info!("Downloaded sources for '{profile}'");
        Ok(())
    }
}
