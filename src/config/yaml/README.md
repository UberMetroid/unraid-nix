# config::yaml

Hand-written YAML serializer and parser, scoped to the small schema
`process-compose.yml` actually uses (the `ProcessComposeConfig` struct
in the parent module). The split is: `value.rs` defines the in-memory
YAML model; `scalar.rs` handles string/quoting; `convert.rs` maps
between `serde_yaml`-style values and our types; `serialize.rs` emits
the file; `parse.rs` reads it; `common.rs` holds shared utilities.
Replaces `serde_yml` to drop the `libyml` C dependency — keep the
output format stable or `process-compose` (the external CLI) will
reject the file.
