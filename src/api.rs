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



#[derive(Debug, Clone)]
struct HostAddr {
    interface: String,
    ip: String,
}

/// Resolves active IPv4 addresses on Unraid host interfaces (excluding loopback and virtual docker/veth interfaces).
fn get_host_ips() -> Vec<HostAddr> {
    let mut ips = Vec::new();
    let output = std::process::Command::new("ip")
        .args(&["-o", "-4", "addr", "show"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let iface = parts[1];
                    let ip_net = parts[3];
                    
                    let iface_lower = iface.to_lowercase();
                    if iface_lower == "lo" || 
                       iface_lower.starts_with("veth") || 
                       iface_lower.starts_with("docker") || 
                       iface_lower.starts_with("br-") || 
                       iface_lower.starts_with("virbr") ||
                       iface_lower.starts_with("shim") {
                        continue;
                    }

                    if let Some(pos) = ip_net.find('/') {
                        let ip = &ip_net[..pos];
                        if !ip.starts_with("127.") {
                            ips.push(HostAddr {
                                interface: iface.to_string(),
                                ip: ip.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push(HostAddr {
            interface: "lo".to_string(),
            ip: "127.0.0.1".to_string(),
        });
    }
    ips
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
    let host_ips = get_host_ips();

    let mut html = r#"<div style="overflow-x: auto; width: 100%;">
        <table class="nix-services-table">
            <thead>
                <tr>
                    <th>Application</th>
                    <th>Version</th>
                    <th>LAN IP:Port</th>
                    <th>Volume Mappings (App to Host)</th>
                    <th>Autostart</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>"#.to_string();

    if statuses.is_empty() {
        html.push_str(r#"<tr><td colspan="6" class="text-center">No Nix Flake services configured. Go to the Flakes tab to install one.</td></tr>"#);
    } else {
        for s in statuses {
            let is_running = s.status.to_lowercase() == "running";

            let status_subtext = if is_running {
                r#"<span style="color: #2ecc71;">●</span> started"#
            } else if s.status.to_lowercase() == "stopped" {
                r#"<span style="color: #e74c3c;">●</span> stopped"#
            } else {
                r#"<span style="color: #f1c40f;">●</span> failed"#
            };

            let app_html = format!(
                r#"<div style="display: flex; flex-direction: column; gap: 2px;">
                    <strong>{}</strong>
                    <span style="font-size: 11px; color: #a0a0a5;">{}</span>
                </div>"#,
                s.name, status_subtext
            );

            let version_html = r#"<span style="color: #2ecc71; font-weight: 500;">up-to-date</span>"#;
            let port_num = get_service_web_port(&s.name);

            let lan_ip_port_html = if let Some(port) = port_num {
                let mut ip_links = Vec::new();
                for addr in &host_ips {
                    let label = match addr.interface.to_lowercase().as_str() {
                        "tailscale0" | "tailscale" => "tailscale".to_string(),
                        other => other.to_string(),
                    };
                    
                    let link = if is_running {
                        format!(
                            r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a> <span style="font-size: 10px; color: #777; font-family: monospace;">({})</span></div>"##,
                            addr.ip, port, addr.ip, port, label
                        )
                    } else {
                        format!(
                            r##"<div style="margin-bottom: 4px;"><span style="color: #888;">{}:{}</span> <span style="font-size: 10px; color: #555; font-family: monospace;">({})</span></div>"##,
                            addr.ip, port, label
                        )
                    };
                    ip_links.push(link);
                }
                ip_links.join("")
            } else {
                "-".to_string()
            };

            let home_path = config
                .as_ref()
                .and_then(|c| c.processes.get(&s.name))
                .map(|p| extract_home_path(&p.command))
                .unwrap_or_else(|| "-".to_string());

            let volume_mappings_html = if home_path != "-" && !home_path.is_empty() {
                format!(
                    r#"<span style="color: #a0a0a5;">/config</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code>"#,
                    home_path
                )
            } else {
                "-".to_string()
            };

            let autostart_enabled = config
                .as_ref()
                .and_then(|c| c.processes.get(&s.name))
                .and_then(|p| p.availability.as_ref())
                .map(|a| a.restart.to_lowercase() == "always")
                .unwrap_or(true);

            let autostart_checked = if autostart_enabled { "checked" } else { "" };
            let autostart_html = format!(
                r#"<label class="nix-switch">
                    <input type="checkbox" onchange="toggleAutostart('{}', this.checked)" {}>
                    <span class="nix-slider"></span>
                </label>"#,
                s.name, autostart_checked
            );

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
                    <td>{}</td>
                    <td>
                        <div class="nix-actions-wrapper">
                            {}
                            {}
                            {}
                        </div>
                    </td>
                </tr>"#,
                app_html, version_html, lan_ip_port_html, volume_mappings_html, autostart_html, start_btn, stop_btn, logs_btn
            ));
        }
    }

    html.push_str("</tbody></table></div>");
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

    let mut html = r#"<div style="overflow-x: auto; width: 100%;">
        <table class="nix-search-table">
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

    html.push_str("</tbody></table></div>");
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
