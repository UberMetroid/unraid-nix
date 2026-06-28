# util

Cross-cutting helpers used by every other module. `mod.rs` re-exports
the submodules; `process.rs` provides `run_with_timeout` and
`run_with_timeout_status`, which bound the wall-clock time spent on
subprocesses (the codebase never calls `Command::output()` or
`.status()` directly, since nix-daemon, NFS mounts, and the
`/usr/local/emhttp/plugins/nix/scripts/*` helpers can block
indefinitely).
