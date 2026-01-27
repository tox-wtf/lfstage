#![allow(clippy::expect_used)]

use std::{
    fs::{self, File}, io::{
        self,
        BufRead, Write,
    }, path::Path, process::{
        Command,
        Stdio,
        exit,
    }, thread
};

use crate::{
    config::CONFIG, profile::Profile
};

// TODO: Create a thiserror for script failures prolly

// This could be written to take environment variables as vector argument but I cba
/// # WARN: MUST CALL A SCRIPT, NOT A COMMAND
#[allow(clippy::panic)]
pub fn exec<R, P>(profile: Option<R>, script: P) -> io::Result<()>
where
    R: AsRef<Profile>,
    P: AsRef<Path>,
{
    let script = script.as_ref();
    if !script.exists() {
        error!("Script: '{}' does not exist", script.display());

        #[cfg(test)]
        panic!("Nonexistent script");

        #[cfg(not(test))]
        exit(1)
    }

    if let Some(profile) = profile {
        let profile = profile.as_ref();
        let base_env = profile.envs_dir().join("base.env");

        if ! base_env.exists() {
            error!("Base environment '{}' does not exist.", base_env.display());
            error!("Refusing to execute commands without a defined environment.");
            exit(1)
        }

        fs::copy("/usr/lib/lfstage/envs/internal.env", "/tmp/lfstage/bashenv")?;

        let mut f = File::options().append(true).open("/tmp/lfstage/bashenv")?;

        let appended_env = format!(
"export ENVS={envs_dir}
export SCRIPTS={scripts_dir}
export JOBS={jobs}
export LFSTAGE_PROFILE={profile}
export LFSTAGE_VERSION={version}
source {rcfile} || exit 2",
            envs_dir = profile.envs_dir().display(),
            scripts_dir = profile.scripts_dir().display(),
            jobs = &CONFIG.jobs,
            rcfile = base_env.display(),
            profile = &profile.name,
            version = env!("CARGO_PKG_VERSION")
        );

        f.write_all(appended_env.as_bytes())?;
    }

    let mut child = Command::new("bash")
        .env_clear()
        .arg("--noprofile")
        .arg("--norc")
        .arg(script.as_os_str())
        .env("BASH_ENV", "/tmp/lfstage/bashenv")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Handle present");
    let stderr = child.stderr.take().expect("Handle present");

    let stdout_thread = thread::spawn(move || {
        let reader = io::BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            trace!("{line}");
        }
    });

    let stderr_thread = thread::spawn(move || {
        let reader = io::BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            debug!("{line}");
        }
    });

    let status = child.wait()?;
    if !status.success() {
        error!("Command failed: {status}");
        return Err(io::Error::other(format!(
            "Command failed: {status}"
        )));
    }

    stdout_thread.join().expect("Handle already joined");
    stderr_thread.join().expect("Handle already joined");

    Ok(())
}

#[macro_export]
macro_rules! exec {
    // Pattern: profile and a script
    ($profile:expr; $script:expr) => {{
        use std::path::Path;
        debug!(
            "Using profile '{}' to execute script '{}'",
            $crate::profile::Profile::new($profile),
            Path::new($script).display(),
        );
        $crate::utils::cmd::exec(Some($profile), $script)
    }};

    // Pattern: just a script
    ($script:expr) => {{
        use std::path::Path;
        use $crate::profile::Profile;

        debug!(
            "Executing {} without a profile",
            Path::new($script).display(),
        );
        $crate::utils::cmd::exec::<&Profile, _>(None, $script)
    }};
}

#[cfg(test)]
mod test {
    use crate::profile::Profile;

    #[test]
    fn exec_no_profile() { assert!(exec!("s"; "/usr/lib/lfstage/scripts/testing.sh").is_ok()) }

    #[test]
    #[should_panic(expected = "Nonexistent script")]
    fn exec_nonexistent_script() { assert!(exec!(Profile::new("testing"); "cat /usr").is_err()) }

    #[test]
    fn exec_pass_reqs() {
        assert!(
            exec!(
                Profile::new("testing");
                "/usr/lib/lfstage/scripts/reqs.sh"
            )
            .is_ok()
        );
    }

    #[test]
    fn exec_ensure_shell_options() {
        assert!(
            exec!(
                Profile::new("testing");
                "/usr/lib/lfstage/scripts/testing.sh"
            )
            .is_ok()
        );
    }

    // #[test]
    // fn exec_script_fails() {
    //     assert!(
    //         exec!(
    //             "x86_64-glibc-tox-stage2";
    //             "/var/lib/lfstage/profiles/x86_64-glibc-tox-stage2/scripts/06-chapter6.sh",
    //         )
    //         .is_err()
    //     );
    // }
}
