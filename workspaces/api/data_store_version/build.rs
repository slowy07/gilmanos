use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // check for environment variabel "SKIP_README".
    // skip README generation
    if env::var_os("SKIP_README").is_some() {
        return;
    }

    let mut source = File::open("src/lib.rs").unwrap();
    let mut template = File::open("README.tpl").unwrap();

    let content = cargo_readme::generate_readme(
        &PathBuf::from("."), //root
        &mut source,
        Some(&mut template),
        true, // add title
        false, // add badges
        false, // add license
        true, // indent headings
    ).unwrap();

    let mut readme = File::create("README.md").unwrap();
    readme.write_all(content.as_bytes()).unwrap();
}