# cli::sandbox

Builds and prints the bubblewrap command line that would isolate a given
service. `mod.rs` re-exports `build::sandbox_cmd`; `build.rs` translates
`SandboxArgs` into a `SandboxConfig` and runs it through
`sandbox::build_bwrap_command`, writing the resulting shell command to stdout
so it can be inspected without executing it.
