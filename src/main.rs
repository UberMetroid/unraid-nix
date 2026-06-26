/// Nix Helper CLI Entrypoint
///
/// This is the main router for the unraid-nix compiled Rust binary.
/// It parses arguments from standard environment variables, handles
/// routing to respective modules, and outputs JSON or HTML payloads.
use std::env;

mod api;
mod config;
mod process;
mod sandbox;
mod store;
mod search;
mod cli;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        cli::print_usage();
        return;
    }
    cli::run(args);
}
