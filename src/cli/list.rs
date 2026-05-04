// cli/list.rs

use std::fs;
use std::path::Path;

use clap::Args;

use super::CmdError;

#[derive(Args, Debug)]
pub struct Cmd {
    /// The profile to list
    ///
    /// If empty, all profiles are listed, though in less detail
    pub profile: Option<String>,
}

impl Cmd {
    // TODO: Refactor, making use of the profile struct here once it exists
    pub fn run(&self) -> Result<(), CmdError> {
        match &self.profile {
            | Some(p) => {
                let profile_path = Path::new("/var/lib/lfstage/profiles").join(p);
                if profile_path.exists() {
                    println!("{p} at {} exists", profile_path.display());
                } else {
                    println!("{p} at {} does not exist", profile_path.display());
                }
            },
            | None => {
                let all_profiles = fs::read_dir("/var/lib/lfstage/profiles")?
                    .map_while(Result::ok)
                    .map(|p| p.path())
                    .filter(|p| p.is_dir())
                    .collect::<Vec<_>>();

                println!("Available profiles:");
                for profile_path in all_profiles {
                    #[allow(clippy::expect_used)]
                    let profile = profile_path.file_name().expect("Profile should have a name").to_string_lossy();
                    println!("{profile} at {}", profile_path.display());
                }
            },
        }

        Ok(())
    }
}
