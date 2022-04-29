fn main() {
    let Err(e) = buildsys::build_imge() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
