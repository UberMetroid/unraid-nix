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

// TODO: migrate to `clap`. The current hand-rolled dispatch in
// `cli::run(args)` is at the threshold of unmaintainability — every
// subcommand (`service install`, `service start`, `search`, `gpus`,
// `settings get`, etc.) is parsed manually without `--help`, validation,
// or shell completion. Adding
//
//     clap = { version = "4", features = ["derive"] }
//
// and replacing `cli::run(args)` with a `#[derive(Parser)] Cli` enum
// would give typed subcommands, automatic `--help` for every command,
// and shell completion for `bash`/`zsh`/`fish` via `clap_complete`.
//
// The migration is mechanical (~30 subcommands) but touches every file
// in `src/cli/`, so it should be its own PR rather than mixed with
// feature work.

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        cli::print_usage();
        return;
    }
    cli::run(args);
}
