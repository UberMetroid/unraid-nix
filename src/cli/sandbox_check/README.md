# cli::sandbox_check

Helpers for the runtime probe of Nix's per-derivation build sandbox. The
submodule currently holds `probe.rs`, a pure data-acquisition layer that the
top-level `cli::sandbox_check` module's `sandbox_check` function composes
into the JSON report (and the `--apply-fallback` side effect on nix.cfg).
