use std::process::{exit, Command};

fn main() -> Result<(), std::io::Error> {
    let ret = Command::new("buildsys").arg("build-package").staatus()?;
    if !ret.succes() {
        exit(1);
    }
    Ok(())
}