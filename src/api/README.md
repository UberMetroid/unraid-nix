# api

Top-level wiring for the WebUI rendering layer. `mod.rs` declares each
submodule and re-exports the public entry points consumed by the PHP
shim: `render_services_table`, `render_search_results`,
`render_dashboard_widget` / `_rows` / `_json`, `dashboard_diff`,
`render_verification_report`, and `render_presets_store`. The
submodules own the actual HTML generation: `services` and
`services_row` for the table, `dashboard` for the tile, `presets_store`
for the install page, `search` and `package` for the search pages,
`report` for the verification report, and `utils` for shared string
helpers and the service-icon resolver.
