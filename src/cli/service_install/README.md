# cli::service_install

Installs a Nix-based service end-to-end: resolves the package, configures
the bubblewrap sandbox, writes the process-compose entry plus per-service
metadata, and starts the supervisor. `mod.rs` defines `ExtraBind` and the
`install_service` orchestrator; `setup.rs` handles appdata directory
creation and extra-binds parsing; `config_writer.rs` writes the
`process-compose.yml` block and sidecar JSON for the new service.
