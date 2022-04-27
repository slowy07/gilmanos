/* !
this module handles the cllss to the buildkit serber needed to execute package and image 
builds. the actul build step and the expected parameters are defined in the repository
to level dockerfile
*/

pub(crate) mod error;
use error::Result;

use duct::cmd;
use rand::Rng;
use snafu::ResultExt;
use std::env;
use std::process::Output;
use users:get_effective_uid;

pub(crate) struct PackageBuilder;

impl PackageBuilder {
    /// Call `buildctl` to produce RPMs for the specified package.
    pub(crate) fn build(package: &str) -> Result<(self)> {
        let arch = getenv("BUILDSYS_ARCH")?;
        let opts = format!(
            "--opt target=rpm \
            --opt build-arg:Package={package} \
            --opt build-arg:Arch={arch}",
            package = package,
            arch = arch,
        );
        let result = buildctl(&opts)?;
        if !result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            return error::PackageBuild {package, output}.fail();
        }
        Ok(self)
    }
}

pub(crate) struct ImageBuilder;

impl ImgeBuilder {
    /// Call `buildctl` to produce RPMs for the specified package.
    pub(crate) fn build(packages: &[string]) -> Result<(self)> {
        let packages = packages.join("|");
        let arch = getenv("BUILDSYS_ARCH")?;
    }
}