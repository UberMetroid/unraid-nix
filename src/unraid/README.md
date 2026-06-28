# unraid

Unraid-OS integration: parsing of Unraid's INI files and the well-known
plugin paths. The module is structured as a top-level `unraid.rs` (in
`src/unraid.rs`, the module file) plus a `unraid/` subdirectory:
`parse.rs` implements the cached `parse_ini_file` reader used for
`nix.cfg`, `shares.cfg`, etc.; `detect.rs` resolves the default Nix
store path from the system share config. Shared constants like
`NIX_CFG_PATH`, `METADATA_DIR`, `PROCESS_COMPOSE_CONFIG`, and
`SUPERVISOR_PORT` are defined in the module file.
