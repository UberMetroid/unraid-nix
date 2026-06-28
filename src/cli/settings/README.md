# cli::settings

Persists WebUI settings to `nix.cfg` and migrates data between store paths.
`mod.rs` exposes `save_settings` (the main writer); `helpers.rs` holds
shared file-system utilities like `has_files`; `migration.rs` performs the
`rsync` from an old store path to a new one when the persistent path
changes. Reads always go through `unraid::parse_ini_file` so the
in-memory model matches the on-disk INI.
