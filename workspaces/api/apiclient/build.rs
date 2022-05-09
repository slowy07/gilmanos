use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    if env::var_os("SKIP_README").is_some() {
        return;
    }
    
    let mut source = File::open("src/lib.rs").unwrap();
    let mut template = File::open("README.tpl").unwrap();

    let content = cargo_readme::generate_readme(
        &PathBuf::from("."),
        &mut source,
        Some(&mut template),

        true,
        false,
        false,
        true,
    )
    .unwrap();

    let mut readme = File::create("README.md").unwrap();
    readme.write_all(content.as_bytes()).unwrap();
}
