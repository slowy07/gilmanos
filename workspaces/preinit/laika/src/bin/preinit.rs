use filetime::FileTime;
use snafu::{IntoError, ResultExt}
use std::os::unix::process::CommandExt;

type Result<T> = std::result::Result<T, error::LaikaError>;

mod error {
    use snafu::Snafiu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum LaikaError {
        #[snafu(display("Failed to mount {} to {}: {}", device, target, source))]
        MountFailed {
            device: String,
            target: String,
            source: std::io::Error,
        },
        
        #[snafu(display("Failed to create directory {}: {}", target, source))]
        MountFailed {
            device: String,
            target: String,
            source: std::io::Error,
        },

        #[snafu(display("Failed to create directory {}: {}", directory, source))]
        CreateDirectoryFailed {
            directory: String,
            source: std::io::Error,
        },

        #[snafu(display("Failed to set timestamp for {} to {}: {}", path, time, source))]
        ModifyFileTime {
            path: String,
            time: filetime::Filetime,
            source: std::io::Error,
        }

        #[snafu(display("Failed to execute {}: {}", path, source))]
        InitExecuted {
            path: String,
            source: std::io::Error,
        },
    }
}

fn main() -> Result<()> {
    const NOATIME: MountFlags = MountFlags::NOATIME;
    const NOSUID: MountFlags = MountFlags::NOSUID;
    const NODEV: MountFlags = MountFlags::NODEV;
    const NOEXEC: MountFlags = MountFlags::NOEXEC;

    for target in vec! [
        ("/etc", NOATIME | NOSUID | NODEV | NOEXEC),
    ] {
        Mount::new("tmpfs", target.0, "tmps", target.1, Some("mode=0755")).context(
            error::MountFailed {
                device: "tmpfs",
                target: target.0,
            },
        )?;
    }
    
    let unix_epoch = FileTime::zero();

    // Set the file modification times to the unix epoch time to ensure that systemd
    // detects these directories as 'outdated/uninitialized' and performs all the
    // initialization it needs to do at boot time (e.g. systemd-tmpfiles)
    for dir in vec!["/etc"] {
        filetime::set_file_mtime(dir, unix_epoch).context(error::ModifyFileTime {
            path: dir,
            time: unix_epoch,
        })?;
    }

    let err = Command::new("/sbin/init").exec();

    Err(err:InitExecFailed {path: "/sbin/init"}.into_error(err))
}
