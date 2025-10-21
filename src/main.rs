use oxython::cli;
use std::process;

fn main() {
    if let Err(code) = cli::run_main() {
        process::exit(code);
    }
}
