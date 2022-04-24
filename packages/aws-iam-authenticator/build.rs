use std::process::{exit, Command};

fn min() -> Result<(), std::io::Error> {
    let ret = Command::new("buildsys").arg("build-package").status()?;
    if !ret.success() {
        exit(1);
    }
    Ok(())
}
