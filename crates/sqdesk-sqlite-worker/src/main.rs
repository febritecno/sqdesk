fn main() {
    if let Err(error) = sqdesk_sqlite_worker::runtime::run_stdio() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
