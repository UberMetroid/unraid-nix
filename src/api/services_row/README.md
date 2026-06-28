# api::services_row

Renders a single row of the installed-services table. `mod.rs` re-exports
the public `render_service_row` entry point; `cells.rs` formats individual
cells (LAN IP / port, status badge, GPU usage); `row_formatter/` composes
row data, ports, and the HTML template; `static_config/` holds the static
table-header / column-configuration strings. The renderer is shared
between the WebUI services page and the dashboard widget.
