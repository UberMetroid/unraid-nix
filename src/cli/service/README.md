# cli::service

Lifecycle actions for installed Nix services and the install helpers that wire
them into the package set. `lifecycle.rs` exposes `service_action`, `autostart`
toggle, the `remove` flow, and supervisor reload; `install.rs` hosts the
`install` (raw nix package) and `preset` (predefined service) CLI commands.
The module is the user-facing entry point for every per-service mutating
operation triggered from `nix-helper`.
