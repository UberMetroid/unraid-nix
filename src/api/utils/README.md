# api::utils

Small string and discovery helpers shared by the rendering layer.
`mod.rs` defines `HostAddr`, the `html_escape` / `js_escape` primitives,
and host-LAN-IP discovery via `get_host_ips`; `icons.rs` resolves the
service-icon path by inspecting each process's `/nix/store/...` binary
in `process-compose.yml`. These are pure helpers — no I/O beyond reading
the config and probing local interfaces.
