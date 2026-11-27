fn main() {
    if let Err(error) = rewrite_internet::github::cli(std::env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
