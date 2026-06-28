# config

`process-compose.yml` data model and persistence. `mod.rs` defines the
`ProcessComposeConfig` struct (the canonical schema for the file
process-compose reads), plus `LogConfiguration` and the per-process
`ProcessConfig`; `file.rs` exposes `load_config` / `save_config` with
the default fallback used when the file is missing; `presets.rs`
resolves the on-disk preset path and renders the per-service sandbox
command. The `yaml/` submodule owns the hand-written emitter and
parser; do not swap it for `serde_yaml` without re-reading the CVE
notes there.
