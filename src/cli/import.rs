// cli/import.rs

use std::fs::write;

use clap::Args;
use fshelpers::mkdir_p;

use crate::exec;

#[derive(Args, Debug)]
pub struct Cmd {
    /// The path to the profile tarball to import
    ///
    /// TODO: Also support tarball urls and github repos
    pub r#in: String,

    /// Whether to perform a dry-run
    #[arg(short, long)]
    pub dry: bool,
}

impl Cmd {
    pub fn run(&self) -> Result<(), super::CmdError> {
        let input = &self.r#in;
        if self.dry {
            println!("Would run /usr/lib/lfstage/scripts/import.sh with import '{input}'");
            return Ok(())
        }

        mkdir_p("/tmp/lfstage")?;
        write("/tmp/lfstage/import", input)?;
        exec!("/usr/lib/lfstage/scripts/import.sh")?;

        info!("Imported profile from '{input}'");
        println!("Imported profile from '{input}'");

        Ok(())
    }
}
