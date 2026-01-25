// cli/build.rs

use std::{
    fs,
    path::Path,
    process::exit,
};

use clap::Args;
use fshelpers::mkdir_p;

use super::{
    CmdError,
    clean::clean_lfs,
};
use crate::{
    config::CONFIG,
    exec,
    profile::Profile,
    utils::time::timestamp,
};

#[derive(Args, Debug)]
pub struct Cmd {
    #[arg(default_value = CONFIG.default_profile.as_str())]
    pub profile: String,

    /// The absolute path to save the stagefile to
    pub stagefile: Option<String>,

    /// Don't actually do anything
    #[arg(short, long)]
    pub dry: bool,

    /// Don't strip all binaries
    ///
    /// All libraries and executables get stripped with --strip-unneeded
    #[arg(short, long)]
    pub skip_strip: bool,

    /// Don't check system requirements
    #[arg(long)]
    pub skip_reqs: bool,
}

impl Cmd {
    /// # Runs the build subcommand
    ///
    /// The build subcommand builds a stage file and accepts a variety of arguments.
    ///
    /// # Arguments
    /// * `self.profile`    - The profile to build, defaults to "x86_64-glibc-tox".
    /// * `self.stagefile`  - The path to the built stagefile, defaults to
    ///   "/var/cache/lfstage/stages/lfstage-<profile>-<timestamp>.tar.xz".
    /// * `self.dry`        - If true, perform a dry run, building nothing.
    ///
    /// * `self.skip_reqs`  - Don't check system requirements
    /// * `self.skip_strip` - Don't strip binaries
    ///
    /// # Errors
    /// This function returns a `CmdError` if:
    /// - The script directory couldn't be read.
    /// - One of the scripts failed.
    pub async fn run(&self) -> Result<(), CmdError> {
        let profile = Profile::new(&self.profile);
        let timestamp = timestamp();

        // Get the path to which the stage file should be saved. Can be overridden if the stagefile
        // positional argument is set.
        let stagefile = match &self.stagefile {
            | Some(path) => path.clone(),
            | None => format!(
                "/var/cache/lfstage/profiles/{profile}/stages/lfstage-{profile}-{timestamp}.tar.xz",
            ),
        };

        // Write some variables to files in `profile_tmpdir` to be accessed later:
        // * `timestamp`    - The timestamp is written to `timestamp`
        // * `stagefile`    - The name of the stagefile is written to `stagefilename`
        // * `strip`        - If we're stripping, create the file `strip`
        if !self.dry {
            // set up `profile_tmpdir`
            mkdir_p(profile.tmpdir())?;

            // timestamp
            fs::write(profile.timestampfile(), &timestamp)?;

            // stagefilename
            fs::write(profile.stagefilenamefile(), &stagefile)?;

            // strip
            if !self.skip_strip && CONFIG.strip {
                fshelpers::mkf(profile.tmpdir().join("strip"))?;
            }
        }

        // The directory for profile-specific scripts
        let scriptdir = &profile.scriptdir();

        // Display what would be done
        if self.dry {
            println!(
                "Would build profile '{profile}' and save it to '{stagefile}' by executing scripts in '{}' and '/usr/lib/lfstage/scripts/'",
                scriptdir.display(),
            );
            return Ok(())
        }

        // Check requirements
        if !self.skip_reqs {
            check_reqs(&profile);
        }

        // TODO: Add profile-specific reqs.sh support

        // Prepare for the build by cleaning and copying over sources
        clean_lfs()?;
        profile.download_sources(false).await?;
        profile.setup_sources()?;

        // Build
        profile.run_build_scripts();

        // TODO: Add signing. Write lfstage metadata to /etc/lfstage-release before saving.

        // Save the stage file
        profile.save_stagefile()?;

        Ok(())
    }
}

fn check_reqs(profile: &Profile) {
    if Path::new("/tmp/lfstage/reqs").exists() {
        return
    }

    if let Err(e) = exec!(&profile; "/usr/lib/lfstage/scripts/reqs.sh") {
        error!("System does not meet requirements: {e}");
        // warn!("If you'd like to continue regardless, `touch /tmp/lfstage/reqs`");
        exit(1)
    }
}
