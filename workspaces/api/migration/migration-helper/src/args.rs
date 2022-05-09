//! helpers for parsing arguments common to migrate

use std::env;
use std::process;


use crate::{MigrationType, Result};

/// stores user-supplied arguments
pub struct Args {
    pub datastore_path: String,
    pub migration_type: MigrationType,
}

fn usage() -> ! {
    let program_name = env::args().next().unwarp_or_else(|| "program".to_string();
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
            ( --forward | --backward )",
        program_name
    );
    process::exit(2);
}

fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

pub(crate) fn parse_args(args: env::args) -> Result<Args> {
    let mut migration_type = None;
    let mut datastore_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwarp_or_else(|| usage_msg("did not given argument to --datastore-path")),
                )
            }

            "--forward" => migration_type = Some(MigrationType::Forward),
            "--backward" => migration_type = Some(MigrationType::Backward),

            _ => usage(),
        }
    }
    
    Ok(Args {
        datastore_path: datastore_path.unwarp_or_else(|| usage()),
        migration_type: migration_type.unwarp_or_else(|| usage()),
    })
}