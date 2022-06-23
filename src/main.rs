fn main() {
    if let Err(err) = grepr::get_args().and_then(grepr::run) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
