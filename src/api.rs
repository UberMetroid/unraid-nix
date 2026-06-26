/// Nix WebGUI HTML/JSON Rendering Module
///
/// This module generates the raw HTML components for the Unraid WebGUI.
/// Renders the Services management list, search results, and Dashboard tiles.

use crate::process::{get_services_status, is_supervisor_running};
use crate::search::search_packages;

/// Resolves default web interface ports for known services.
fn get_service_web_port(name: &str) -> Option<u16> {
    let name_lower = name.to_lowercase();
    if name_lower.contains("sonarr") {
        Some(8989)
    } else if name_lower.contains("radarr") {
        Some(7878)
    } else if name_lower.contains("jellyfin") {
        Some(8096)
    } else {
        None
    }
}

/// Extracts the HOME directory path from the runuser/setpriv command.
fn extract_home_path(command: &str) -> String {
    if let Some(pos) = command.find("export HOME=") {
        let start = pos + "export HOME=".len();
        let sub = &command[start..];
        
        let mut end = sub.len();
        for (i, c) in sub.char_indices() {
            if c == ' ' || c == ';' || c == '&' || c == '"' || c == '\'' {
                end = i;
                break;
            }
        }
        let path = sub[..end].trim();
        if !path.is_empty() {
            return path.to_string();
        }
    }
    "-".to_string()
}

/// Helper to recursive sum files size in a directory.
fn get_dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            } else if p.is_dir() {
                total += get_dir_size(&p);
            }
        }
    }
    total
}

/// Helper to format file sizes in bytes to human-readable units.
fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, units[unit_idx])
}

/// Query NVIDIA GPU utilization once per render.
fn get_nvidia_gpu_usage() -> (std::collections::HashMap<i32, u64>, bool) {
    let mut map = std::collections::HashMap::new();
    let output = std::process::Command::new("nvidia-smi")
        .args(&["--query-compute-apps=pid,used_memory", "--format=csv,noheader,nounits"])
        .output();
    
    match output {
        Ok(out) => {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(pid), Ok(mem)) = (parts[0].trim().parse::<i32>(), parts[1].trim().parse::<u64>()) {
                            map.insert(pid, mem);
                        }
                    }
                }
            }
            (map, true)
        }
        Err(_) => (map, false),
    }
}

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

    let config_path = "/boot/config/plugins/nix/process-compose.yml";
    let config = crate::config::load_config(config_path).ok();
    let (gpu_map, has_gpu) = get_nvidia_gpu_usage();

    let mut html = r#"<table class="nix-services-table">
        <thead>
            <tr>
                <th>Service</th>
                <th>Status</th>
                <th>Port(s)</th>
                <th>Resources</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>"#.to_string();

    if statuses.is_empty() {
        html.push_str(r#"<tr><td colspan="5" class="text-center">No Nix Flake services configured. Go to the Flakes tab to install one.</td></tr>"#);
    } else {
        for s in statuses {
            let is_running = s.status.to_lowercase() == "running";

            let status_badge = if is_running {
                r#"<span class="status green">🟢 Running</span>"#
            } else if s.status.to_lowercase() == "stopped" {
                r#"<span class="status red">🔴 Stopped</span>"#
            } else {
                r#"<span class="status yellow">🟡 Failed</span>"#
            };

            let cpu_str = s.cpu.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "-".to_string());
            let mem_str = s.memory.map(|m| format!("{} MB", m / 1024 / 1024)).unwrap_or_else(|| "-".to_string());
            let uptime_str = s.uptime();

            let status_html = if is_running {
                format!(
                    r#"<div style="display: flex; flex-direction: column; gap: 4px; align-items: flex-start;">
                        {}
                        <span style="font-size: 11px; color: #888;">{} uptime</span>
                    </div>"#,
                    status_badge, uptime_str
                )
            } else {
                format!(
                    r#"<div style="display: flex; flex-direction: column; gap: 4px; align-items: flex-start;">
                        {}
                    </div>"#,
                    status_badge
                )
            };

            let port_str = if let Some(port) = get_service_web_port(&s.name) {
                if is_running {
                    format!(
                        r##"<a href="#" onclick="window.open('http://' + window.location.hostname + ':{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 10px;"></i></a>"##,
                        port, port
                    )
                } else {
                    port.to_string()
                }
            } else {
                "-".to_string()
            };

            let home_path = config
                .as_ref()
                .and_then(|c| c.processes.get(&s.name))
                .map(|p| extract_home_path(&p.command))
                .unwrap_or_else(|| "-".to_string());

            let service_html = if home_path != "-" && !home_path.is_empty() {
                format!(
                    r#"<div style="display: flex; flex-direction: column; gap: 4px;">
                        <strong>{}</strong>
                        <span style="font-size: 11px; color: #888; font-family: monospace; word-break: break-all;">{}</span>
                    </div>"#,
                    s.name, home_path
                )
            } else {
                format!("<strong>{}</strong>", s.name)
            };

            let disk_size_str = if home_path != "-" && !home_path.is_empty() {
                let p = std::path::Path::new(&home_path);
                if p.exists() {
                    format_size(get_dir_size(p))
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            };

            let gpu_str = if let Some(pid) = s.pid {
                if let Some(mem) = gpu_map.get(&pid) {
                    format!("{} MB VRAM", mem)
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            };

            let resources_html = if is_running {
                let mut res = format!(
                    r#"<div class="nix-resources-container">
                        <div><strong>CPU</strong> <span>{}</span></div>
                        <div><strong>RAM</strong> <span>{}</span></div>"#,
                    cpu_str, mem_str
                );
                if has_gpu && gpu_str != "-" {
                    res.push_str(&format!("<div><strong>GPU</strong> <span>{}</span></div>", gpu_str));
                }
                res.push_str(&format!(
                    r#"<div><strong>Disk</strong> <span>{}</span></div>
                    </div>"#,
                    disk_size_str
                ));
                res
            } else {
                format!(
                    r#"<div class="nix-resources-container">
                        <div><strong>Disk</strong> <span>{}</span></div>
                    </div>"#,
                    disk_size_str
                )
            };

            let start_btn = if !is_running {
                format!(r#"<button type="button" class="nix-btn" onclick="serviceAction('{}', 'start')" title="Start"><i class="fa fa-play"></i></button>"#, s.name)
            } else {
                "".to_string()
            };

            let stop_btn = if is_running {
                format!(r#"<button type="button" class="nix-btn" onclick="serviceAction('{}', 'stop')" title="Stop"><i class="fa fa-stop"></i></button>"#, s.name)
            } else {
                "".to_string()
            };

            let logs_btn = format!(r#"<button type="button" class="nix-btn" onclick="openLogs('{}')" title="Logs"><i class="fa fa-file-text-o"></i></button>"#, s.name);

            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>
                        <div class="nix-actions-wrapper">
                            {}
                            {}
                            {}
                        </div>
                    </td>
                </tr>"#,
                service_html, status_html, port_str, resources_html, start_btn, stop_btn, logs_btn
            ));
        }
    }

    html.push_str("</tbody></table>");
    html
}

fn is_cli_enabled() -> bool {
    if let Ok(content) = std::fs::read_to_string("/boot/config/plugins/nix/nix.cfg") {
        for line in content.lines() {
            if line.starts_with("ENABLE_CLI_INSTALL=") {
                let val = line.trim_start_matches("ENABLE_CLI_INSTALL=").trim_matches('"');
                return val == "yes";
            }
        }
    }
    false
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
        let cli_enabled = is_cli_enabled();
        for r in results {
            let action_buttons = if cli_enabled {
                format!(
                    r#"<div style="display: flex; flex-direction: column; gap: 5px; align-items: center; justify-content: center;">
                        <button type="button" class="nix-btn" style="width: 100px; margin: 0; padding: 4px 8px; font-size: 11px;" onclick="installPackage('{}')">Install CLI</button>
                        <button type="button" class="nix-btn-install" style="width: 100px; margin: 0; padding: 4px 8px; font-size: 11px;" onclick="showServiceModal('{}')">Add Service</button>
                       </div>"#,
                    r.package_name, r.package_name
                )
            } else {
                format!(
                    r#"<button type="button" class="nix-btn-install" style="width: 100px; margin: 0; padding: 4px 8px; font-size: 11px;" onclick="showServiceModal('{}')">Add Service</button>"#,
                    r.package_name
                )
            };

            let short_name = r.package_name.replace("nixpkgs#", "");
            let package_link = format!(
                r#"<a href="https://search.nixos.org/packages?channel=unstable&query={}" target="_blank" style="color: #00a1ff; text-decoration: none;"><code>{}</code> <i class="fa fa-external-link" style="font-size: 10px; margin-left: 2px;"></i></a>"#,
                short_name, r.package_name
            );

            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                package_link, r.version, r.description, action_buttons
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
