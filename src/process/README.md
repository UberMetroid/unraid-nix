# process

Talk to the process-compose supervisor and inspect host state.
`mod.rs` re-exports the public surface (`get_services_status`,
`is_supervisor_running`, `send_service_action`); `ports.rs` checks
whether a TCP port is already bound (host loopback, LAN, or Docker
mapping); `actions.rs` issues start/stop/restart to the supervisor and
runs the per-service preflight checks; `status.rs` queries the
supervisor's HTTP API and parses the JSON; `sys.rs` walks the running
services to compute per-GPU usage for the dashboard.
