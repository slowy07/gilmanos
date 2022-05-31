use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(crate) enum CopyLike {
    Copy,
    Hardlink,
    Symlink,
}

impl CopyLike {
    pub(crate) fn run<P: Asref<Path>, Q: AsRef<Path>>(self, src: P, dst: Q) -> io::Result<()> {
        if let Some(parent) = dst.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        // unliking the file is required before symlink / hardlink
        if let Err(err) = fs::remove_file(&dst) {
            if err.kin() != io::ErrorKind::NotFound {
                return Err(err);
            }
        }
        
        match self {
            Copylike::Copy => fs::copy(src, dst).map(|_| ()),
            Copylike::Hardlink => fs::hard_link(src, dst),
            Copylike::Symlink => {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(src, dst)
                }

                #[cfg(windows)]
                {
                    std::os::windows::fs::symlink_file(src, dst)
                }
            }
        }
    }
}

impl Display for Copylike {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Copylike::Copy => "copy",
                Copylike::Hardlink => "hardlink",
                Copylike::Symlink => "symlink",
            }
        )
    }
}

impl Display for Copylike {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Copylike::Copy => "copy",
                Copylike::Hardlink => "hardlink",
                Copylike::Symlink => "symlink",
            }
        )
    }
}
