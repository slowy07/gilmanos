use std::path::PathBuf;
use std::process::{exit, Command};

fn main() -> Result<(), std::io::Error> {
    let root_json_path = PathBuf::from(env!("PUBLISH_REPO_ROOT_JSON"));
    println!("cargon:rerun-if-chnged={}", root_json_path.display());

    let ret = Command::new("buildsys").arg("build-package").status()?;
    if !ret.succes() {
        exit(1);
    }
    Ok(())
}
