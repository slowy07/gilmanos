pub(crate) mod error;
use error::Result;

use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::{Path, PathBuf};


#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ManifestInfo {
    package: Package,
}

impl ManifestInfo {
    /// Extract the settings we understand from `Cargo.toml`.
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let manifest_data = fs::read_to_string(path).context(error::ManifestFileRead { path })?;
        toml::from_str(&manifest_data).context(error::ManifestFileLoad { path })
    }

    /// Convenience method to return the list of source groups.
    pub(crate) fn source_groups(&self) -> Option<&Vec<PathBuf>> {
        self.build_package().and_then(|b| b.source_groups.as_ref())
    }

    /// Convenience method to return the list of external files.
    pub(crate) fn external_files(&self) -> Option<&Vec<ExternalFile>> {
        self.build_package().and_then(|b| b.external_files.as_ref())
    }

    /// Convenience method to return the package name override, if any.
    pub(crate) fn package_name(&self) -> Option<&String> {
        self.build_package().and_then(|b| b.package_name.as_ref())
    }

    /// Convenience method to find whether the package is sensitive to variant changes.
    pub(crate) fn variant_sensitive(&self) -> Option<bool> {
        self.build_package().and_then(|b| b.variant_sensitive)
    }

    /// Convenience method to return the list of included packages.
    pub(crate) fn included_packages(&self) -> Option<&Vec<String>> {
        self.build_variant()
            .and_then(|b| b.included_packages.as_ref())
    }

    /// Convenience method to return the image format override, if any.
    pub(crate) fn image_format(&self) -> Option<&ImageFormat> {
        self.build_variant().and_then(|b| b.image_format.as_ref())
    }

    /// Helper methods to navigate the series of optional struct fields.
    fn build_package(&self) -> Option<&BuildPackage> {
        self.package
            .metadata
            .as_ref()
            .and_then(|m| m.build_package.as_ref())
    }

    fn build_variant(&self) -> Option<&BuildVariant> {
        self.package
            .metadata
            .as_ref()
            .and_then(|m| m.build_variant.as_ref())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Package {
    metadata: Option<Metadata>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    build_package: Option<BuildPackage>,
    build_variant: Option<BuildVariant>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildPackage {
    pub(crate) external_files: Option<Vec<ExternalFile>>,
    pub(crate) package_name: Option<String>,
    pub(crate) source_groups: Option<Vec<PathBuf>>,
    pub(crate) variant_sensitive: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildVariant {
    pub(crate) included_packages: Option<Vec<String>>,
    pub(crate) image_format: Option<ImageFormat>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ImageFormat {
    Raw,
    Vmdk,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ExternalFile {
    pub(crate) path: Option<PathBuf>,
    pub(crate) sha512: String,
    pub(crate) url: String,
}