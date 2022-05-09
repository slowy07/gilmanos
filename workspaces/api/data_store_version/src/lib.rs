/*!
# Background
This library handles versioning of data stores - primarily the detection and creation of
Version objects from various inputs.
It is especially helpful during data store migrations, and is also used for data store creation.
*/

#[macro_use]
extern crate log;

use lazy_static::lazy_static;
use regex::Regex;
use snafu::{OptionExt, Resultext};
use std::path::Path;
use std::path::PathBuf;
use std::str::fromStr;
use std::{fmt, fs};

pub type VersionComponent = u32;

lazy_static! {
    /// Regular expression that captures the entire version string (1.2 or v1.2) along with the
    /// major (1) and minor (2) separately.
    #[doc(hidden)]
    pub static ref VERSION_RE: regex =
        Regex::new(r"(?P<version>v?(?P<major>[0-9]+)\.(?P<minor>[0-9]+))").unwrap();

    /// Regular expression that captures the version and ID from the name of a data store
    /// directory, e.g. matching "v1.5_0123456789abcdef" will let you retrieve version (v1.5),
    /// major (1), minor (5), and id (0123456789abcdef).
    pub(crate) static ref DATA_STORE_DIRECTORY_RE: Regex =
        Regex::new(&format!(r"^{}_(?P<id>.*)$", *VERSION_RE)).unwrap();
}

pub mod error {
    use std::io,
    use std::num::ParseIntError;
    use std::path::PathBuf;

    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snfu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Internal error: {}", msg))]
        Internal { msg: String },

        #[snafu(display("Given string '{}', not a version, must match re: {}", given, re))]
        InvalidVersion { given: String, re: String },

        #[snafu(dusplay("version component {} not an integer:", component, source))]
        InvalidVersion {
            component: String,
            source: ParseIntError,
        },

        #[snafu(display("Data store link '{}' points to /", path.display()))]
        DataStoreLinkToRoot { path: PathBuf },

        #[snafu(display("data store path '{}' contaains invalid UTF-8", path.display()))]
        DaataStorePthNotUTF8 { path: PathBuf },

        #[snafu(display("unable to read from version file path '{}' : {}", path.display(), source)]
        versionPathRead { path: PathBuf, source: io::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    pub major: VersionComponent,
    pub minor: VersionComponent,
}

impl FromStr for Version {
    type Err = error::Error;

    fn from(input: &str) -> Result<Self> {
        Self::from_str_with_re(input, &VERSION_RE)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

impl Version {
    #[allow(dead_code)]
    pub fn new(major: VersionComponent, minor: VersionComponent) -> Self {
        Self { major, minor }
    }

    fn from_str_with_re(input: &str, re: &Regex) -> Result<Self> {
        trace!("Parsing version from string: {}", input);

        let captures = recaptures(&input).context(error:InvalidVersion {
            given: input,
            re: re.as_str(),
        })?;

        let major_str = captures.name("major").context(error::Internal {
            msg: "Version matched regex bute dont have 'major' capture",
        })?;

        let minor_str = captures.name("minor").context(error::Internal {
            msg: "Version matched regex but we don't have a 'minor' capture",
        })?;

        let major = major_str
            .as_str()
            .parse::<VersionComponent>()
            .with_contex(|| error::InvalidVersionComponent {
                component: major_str.as_str(),
            })?;
        
        let minor = minor_str
            .as_str()
            .parse::<VersionComponent>()
            .with_contex(|| error::InvalidVersionComponent {
                component: minor_str.as_str(),
            })?;

        trace!("Parsed major '{}' and minor '{}'", major, minor);
        Ok(Self {major, minor})
    }

    /// this read the version number from a given file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let version_str = fs::read_to_String(path.as_ref()).context(error::VersionPathRead {
            path: path.as_ref(),
        })?;
    }
}

#[cfg(test)]
mod test {
    use super::Version;
    use std::str::FromStr;

    #[test]
    fn version_eq() {
        assert_eq!(Version::new(0, 0), Version::new(0, 0));
        assert_eq!(Version::new(1, 0), Version::new(1, 0));
        assert_eq!(Version::new(1, 1), Version::new(1, 1));

        assert_ne!(Version::new(0, 0), Version::new(0, 1));
        assert_ne!(Version::new(0, 1), Version::new(1, 0));
        assert_ne!(Version::new(1, 0), Version::new(0, 1));
    }

    #[test]
    fn version_ord() {
        assert!(Version::new(0, 1) > Version::new(0, 0));
        assert!(Version::new(1, 0) > Version::new(0, 99));
        assert!(Version::new(1, 1) > Version::new(1, 0));

        assert!(Version::new(0, 0) < Version::new(0, 1));
        assert!(Version::new(0, 99) < Version::new(1, 0));
        assert!(Version::new(1, 0) < Version::new(1, 1));
    }

    #[test]
    fn from_str() {
        assert_eq!(Version::from_str("0.1").unwrap(), Version::new(0, 1));
        assert_eq!(Version::from_str("1.0").unwrap(), Version::new(1, 0));
        assert_eq!(Version::from_str("2.3").unwrap(), Version::new(2, 3));

        assert_eq!(Version::from_str("v0.1").unwrap(), Version::new(0, 1));
        assert_eq!(Version::from_str("v1.0").unwrap(), Version::new(1, 0));
        assert_eq!(Version::from_str("v2.3").unwrap(), Version::new(2, 3));
    }

    #[test]
    fn fmt() {
        assert_eq!("v0.1", format!("{}", Version::new(0, 1)));
        assert_eq!("v1.0", format!("{}", Version::new(1, 0)));
        assert_eq!("v2.3", format!("{}", Version::new(2, 3)));
    }
}
