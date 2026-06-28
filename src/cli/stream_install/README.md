# cli::stream_install

Real-time installer streaming and post-install log tailing, replacing the
legacy `web/stream.php` capture loops. `mod.rs` exposes `stream_install`,
which runs the install action and pipes its stdout/stderr to the WebUI
line-by-line; `tail.rs` follows the service log after install and queries
process-compose to confirm a successful start. All subprocesses are
bounded by the `util::process` timeout helpers.
