# sandbox

Top-level sandbox plumbing shared by the CLI and rendering layers.
`mod.rs` defines `SandboxConfig`, `build_bwrap_command` (the
public entry point), `parse_binds_string`, the POSIX-safe `sh_quote`
helper, and `has_traversal` (the path-traversal guard used to reject
`../` in user-supplied bind mounts). `builder/` holds the two
isolation-strategy implementations; `cli.rs` parses the bind-mount
CLI string into a structured list.
