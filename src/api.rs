/// Nix WebGUI HTML/JSON Rendering Module
///
/// This module generates the raw HTML components for the Unraid WebGUI.
/// Renders the Services management list, search results, and Dashboard tiles.

use crate::process::{get_services_status, is_supervisor_running};
use crate::search::search_packages;

/// Renders the services dashboard table as an HTML string.
/// Mirrors the styling and visual cues of Unraid's native Docker container list.
pub fn render_services_table(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<div class="alert alert-warning"><i class="fa fa-exclamation-triangle"></i> Nix process supervisor (process-compose) is not running. Start the array to launch the services.</div>"#.to_string();
    }

    let statuses = match get_services_status(api_port) {
        Ok(s) => s,
        Err(e) => return format!(r#"<div class="alert alert-danger"><i class="fa fa-times"></i> Error connecting to supervisor API: {}</div>"#, e),
    };

    let mut html = r#"<table class="nix-services-table">
        <thead>
            <tr>
                <th>Service Name</th>
                <th>Status</th>
                <th>Uptime</th>
                <th>CPU</th>
                <th>Memory</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>"#.to_string();

    if statuses.is_empty() {
        html.push_str(r#"<tr><td colspan="6" class="text-center">No Nix Flake services configured. Go to the Flakes tab to install one.</td></tr>"#);
    } else {
        for s in statuses {
            let status_badge = if s.status.to_lowercase() == "running" {
                r#"<span class="status green">🟢 Running</span>"#
            } else if s.status.to_lowercase() == "stopped" {
                r#"<span class="status red">🔴 Stopped</span>"#
            } else {
                r#"<span class="status yellow">🟡 Failed</span>"#
            };

            let cpu_str = s.cpu.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "-".to_string());
            let mem_str = s.memory.map(|m| format!("{} MB", m / 1024 / 1024)).unwrap_or_else(|| "-".to_string());
            let uptime_str = s.uptime.unwrap_or_else(|| "-".to_string());

            html.push_str(&format!(
                r#"<tr>
                    <td><strong>{}</strong></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>
                        <button class="nix-btn" onclick="serviceAction('{}', 'start')"><i class="fa fa-play"></i></button>
                        <button class="nix-btn" onclick="serviceAction('{}', 'stop')"><i class="fa fa-stop"></i></button>
                        <button class="nix-btn" onclick="serviceAction('{}', 'restart')"><i class="fa fa-refresh"></i></button>
                        <button class="nix-btn" onclick="openLogs('{}')"><i class="fa fa-file-text-o"></i></button>
                    </td>
                </tr>"#,
                s.name, status_badge, uptime_str, cpu_str, mem_str, s.name, s.name, s.name, s.name
            ));
        }
    }

    html.push_str("</tbody></table>");
    html
}

/// Renders search results from the Nixpkgs registry into an HTML table.
pub fn render_search_results(query: &str) -> String {
    let results = match search_packages(query) {
        Ok(r) => r,
        Err(e) => return format!(r#"<div class="alert alert-danger"><i class="fa fa-times"></i> Search failed: {}</div>"#, e),
    };

    let mut html = r#"<table class="nix-search-table">
        <thead>
            <tr>
                <th>Package Name</th>
                <th>Version</th>
                <th>Description</th>
                <th>Action</th>
            </tr>
        </thead>
        <tbody>"#.to_string();

    if results.is_empty() {
        html.push_str(r#"<tr><td colspan="4" class="text-center">No packages found matching your query.</td></tr>"#);
    } else {
        for r in results {
            html.push_str(&format!(
                r#"<tr>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>
                        <button class="nix-btn-install" onclick="installPackage('{}')">Install CLI</button>
                        <button class="nix-btn-install" onclick="showServiceModal('{}')">Add Service</button>
                    </td>
                </tr>"#,
                r.package_name, r.version, r.description, r.package_name, r.package_name
            ));
        }
    }

    html.push_str("</tbody></table>");
    html
}

/// Renders the HTML table body for the main Unraid Dashboard widget.
pub fn render_dashboard_widget(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<tr><td colspan="4" class="text-center text-muted">Supervisor is stopped.</td></tr>"#.to_string();
    }

    let mut html = String::new();
    if let Ok(statuses) = get_services_status(api_port) {
        for s in statuses {
            let status_indicator = if s.status.to_lowercase() == "running" {
                r#"<span class="status-dot green"></span>"#
            } else {
                r#"<span class="status-dot red"></span>"#
            };
            let cpu_str = s.cpu.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "-".to_string());
            let mem_str = s.memory.map(|m| format!("{}M", m / 1024 / 1024)).unwrap_or_else(|| "-".to_string());

            html.push_str(&format!(
                r#"<tr>
                    <td>{} {}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                status_indicator, s.name, cpu_str, mem_str
            ));
        }
    } else {
        html.push_str(r#"<tr><td colspan="3" class="text-center">Error reading statuses.</td></tr>"#);
    }
    html
}
