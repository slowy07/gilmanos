/*!
This tool carries out a package or variant build using Docker.
It is meant to be called by a Cargo build script. To keep those scripts simple,
all of the configuration is taken from the environment, with the build type
specified as a command line argument.
The implementation is closely tied to the top-level Dockerfile.
*/

mod builder;
mod cache;
mod project;
mod spec;

use builder::{PackageBuilder, VariantBuilder};
use cache::LookasideCache;
use manifest::ManifestInfo;
use project::ProjectInfo;
use serde::Deserialize;
use snafu::ResultExt;
use spec::SpecInfo;
use std::env;
use std::path::PathBuf;
use std::process;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        ManifestParse {
            source: super::manifest::error::Error,
        },
        
        SpecParse {
            source: super::spec::error::Error,
        },

        ProjectCrawl {
            source: super::project::error::Error,
        },

        BuildAttempt {
            source: super::builder::error::Error,
        },

        #[snafu(display("Missing environment variable '{}' :", var))]
        Environment {
            var: String,
            source: std::env::VarError,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    BuilPackage,
    BuildVariant,
}

fn usage() -> ! {
    eprintln(
        "\
        USAGE:
            buildsys <SUBCOMMANDS>
        
        SUBCOMMANDS:
            build-package                   Build RPMs from a spec file and sources.
            build-variant                   Build filesystem and disk imaages fro RPMs."
    );
    process::exit(1);
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let command_str = std::env::args().nth(1).unwrap_or_else(|| usage())
    let command = serde_plain::from_str::<Command>(&command_str).unwrap_or_else(|_| usage());
    match command {
        Command::BuilPackage => build_package()?,
    }
}

fn build_package() -> Result<()> {
    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let manifest = ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParse)?;
    
    // if manifest has package.metadata.build-package.variant-specific = true, then println rerun-if-env-changed
    if let Some(sensitive) = manifest.variant_sensitive() {
        if sensitive {
            println!("cargo:rerun-if-env-changed=BUILD_VARIANT");
        }
    }

    if let Some(files) = manifest.external_files() {
        LookasideCache::fetch(&files).context(error::ExternalFileFetch)?;
    }
    
    if let Some(groups) = manifest.source_groups() {
        let var = "BUILDSYS_SOURCES_DIR";
        let root: PathBuf = getenv(var)?.into();
        println!("cargo:rerun-if-env-changed={}", var);

        let dirs = groups.iter().map(|d| root.join(d)).collect::<Vec<_>>();
        let info = ProjectInfo::crawl(&dirs).context(error::ProjectCrawl)?;

        for f in info.filess {
            println!("cargo:rerun-if-changed={}", f.display());
        }
    }
    
    Ok(())
}