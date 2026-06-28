# api::dashboard

Renders the Unraid dashboard widget and the per-state diff the widget
consumes. `mod.rs` re-exports `render_dashboard_widget`,
`render_dashboard_rows`, `render_dashboard_json`, and `dashboard_diff`;
`rows.rs` sorts and formats the per-service rows; `widget.rs` wraps the
rows in the `<table>` tile the Unraid dashboard expects; `diff.rs`
compares the current state against a cached JSON snapshot to drive the
"new / changed" badges.
