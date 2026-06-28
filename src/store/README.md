# store

Host-side plumbing for the persistent Nix store and the nixbld users.
`mod.rs` re-exports the public surface (`mount_nix_store`, `unmount_*`,
`create_builder_accounts`, `setup_nix_conf`, plus the config helpers);
`mount.rs` bind-mounts the configured persistent path at `/nix` and
unmounts it on teardown; `setup.rs` wires the persistent `nix.conf` and
`nix.registry.json` symlinks into `/nix/etc`; `accounts.rs` creates the
`nixbld1..N` users in the 30000+ UID range to avoid clashes with Unraid
GUI accounts; `config/` holds the nix.conf generator, reader, and
logger.
