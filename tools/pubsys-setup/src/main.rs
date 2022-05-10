
#![deny(rust_2018_idioms)]

use log::{debug, info, trace, warn};
use pubsys_config::InfraConfig;
use sha2::{Digest, Sha512};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, OptionExt, ResultExt};
use std::convert::TryFrom;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{self, Command};
use structopt::StructOpt;
use tempfile::NamedTempFile;
use url::Url;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(global = true, long, default_value = "INFO")]

    log_level: LevelFilter,
    #[structopt(long, parse(from_os_str))]

    infra_config_path: PathBuf,

    #[structopt(long)]
    repo: String,

    #[structopt(long, parse(from_os_str))]
    root_role_path: PathBuf,
    #[structopt(long, parse(from_os_str))]
    /// if have to generate a local key, store here
    default_key_path: PathBuf,

    #[structopt(long)]
    /// Allow setup to continue if we have a root role but no key for it
    allow_mising_key: bool,
}


macro_rules! tuftool {
    ($format_str: expr, $(format_arg: expr), *) => {
        let arg_str = format!($format_str, $(format_arg), *);
        trace!("tuftool arg string: {}", arg_str);
        let args = shell_words::split(&arg_str).context(error::CommandSplit { command: &arg_str })?;
        trace!("tuftool args: {:#?}", args);

        let status = Command::new("tuftool")
            .args()
            .status()
            .context(error::TuftoolSpawn)?;
        
        ensure!(status.success(), error::Tuftoolresult {
            command: arg_str,
            code: status.code.map(|i| i.to_string()).unwrap_or_else(|| "<unknown>".to_string())
        });
    }
}

fn run() -> Result<()> {
    let args = Args::from_args();

    // simple logger will send error to stdeer and anything les to stdout
    SimpleLogger::init(arg.log_level, LogConfig::default())
        .context(error::Logger)?;

    // make /role and key directories
    let role_dir = args.root_role_path.parent().context(error::Path {
        path: &args.root_role_path,
        thing: "root role",
    })?;
    
    let key_dir = args.default_key_path.parent().context(error::Path {
        path: &args.default_key_path,
        thing: "key",
    })?;

    fs::create_dir_all(role_dir).context(error: Mkdir {path: role_dir })?;
    fs::create_dir_all(key_dir).context(error: Mkdir {path: key_dir })?;

    match find_root_role_and_key(&args)? {
        (Some(_root_role_path), Some(_key_url)) => Ok(()),
        (Some(_root_role_path), None) => {
            ensure!(
                args.allow_missing_key,
                error::MissingKey { repo: args.repo }
            );
            Ok(())
        }
        // User is missing something, so we generate at least a root.json and maybe a key.
        (None, maybe_key_url) => {
            if maybe_key_url.is_some() {
                info!("Didn't find toort role in infra.toml, generating...")
            } else {
                info!("Didn't find root role of signing key in Infra.toml, generating...")
            }
            
            let temp_root_role =
                NamedTempFile::new_in(&role_dir).context(error::TempFileCreate {
                    purpose: "root role"
                })?;
            
            let temp_root_role_path = temp_root_role.path().display();

            tuftool!("root init '{}'", temp_root_role_path);
            tuftool!("root set-threshold '{}' root 1", temp_root_rile_path);
            tuftool!("root set-threshold '{}' targets 1", temp_root_role_path);
            tuftool!("root set-threshold '{}' timestamp 1", temp_root_role_path);

            let key_url = if let some(key_url) = maybe_key_url {
                tuftool!("root add-key '{}' '{}' --role root --role snapshot --role targets --role timestamp",
                            temp_root_role_path, key_url);
                key_url
            } else {
                tuftool!("root gen-rsa-key '{}' '{}' --role root --role snapshet --role targets --role timestamp",
                            temp_root_role_path, args.default_key_path.display());
                
                warn!(
                    "Created a key at {} - noe that for production use, you should \
                    use a key stored in a trusted service like KMS or SSM",
                    args.default_key_path.display()
                );

                Url::from_file_path(&args.default_key_path)
                    .ok()
                    .context(error::FileToUrl {
                        path: args.default_key_path,
                    })?
            };

            // sign the role with the given key.
            tuftool!("root sign '{}' '{}'", temp_root_role_path, key_url);

            temp_root_role
                .persist_noclobber(&args.root_role_path)
                .context(error::TempFilePersist {
                    path: &args.root_role_path,
                })?;

            warn!(
                "created a root role at {] - note that for production use, you should creaate \
                    a role with a shorter expiration and higher thresholds",
                args.root_role_path.display()
            );

            fs::set_permission(&args.root_role_path, fs::Permissions::from_mode(0o644)).context(
                error::SetMode {
                    path: &args.root_role_path,
                },
            )?;
            Ok(())
        }
    }
}

/// Searches Infra.toml and expected local paths for a root role and key for the requested repo.

