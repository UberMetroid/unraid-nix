/// Nix WebGUI HTML/JSON Rendering Module
///
/// This module generates the raw HTML components for the Unraid WebGUI.
/// Renders the Services management list, search results, and Dashboard tiles.

use crate::process::{get_services_status, is_supervisor_running};
use crate::search::search_packages;
use std::collections::HashMap;

/// Resolves default web interface ports for known services.
fn get_service_web_port(name: &str) -> Option<u16> {
    let metadata_path = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(port_val) = val.get("port") {
                    if let Some(num) = port_val.as_u64() {
                        if num > 0 {
                            return Some(num as u16);
                        }
                    }
                    if let Some(s) = port_val.as_str() {
                        if let Ok(num) = s.parse::<u16>() {
                            return Some(num);
                        }
                        let mappings = crate::sandbox::parse_ports(s);
                        if !mappings.is_empty() {
                            let name_lower = name.to_lowercase();
                            if name_lower.contains("jellyfin") {
                                if let Some(m) = mappings.iter().find(|m| m.container == 8096) {
                                    return Some(m.host);
                                }
                            } else if name_lower.contains("syncthing") {
                                if let Some(m) = mappings.iter().find(|m| m.container == 8384) {
                                    return Some(m.host);
                                }
                            }
                            return Some(mappings[0].host);
                        }
                    }
                }
            }
        }
    }

    let name_lower = name.to_lowercase();
    if name_lower.contains("sonarr") {
        Some(8989)
    } else if name_lower.contains("radarr") {
        Some(7878)
    } else if name_lower.contains("jellyfin") {
        Some(8096)
    } else if name_lower.contains("syncthing") {
        Some(8384)
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

/// Extracts the package URI (e.g. nixpkgs#sonarr) from a command string.
fn extract_package_uri(command: &str) -> Option<String> {
    if let Some(pos) = command.find("nixpkgs#") {
        let sub = &command[pos..];
        let end = sub.find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ';')
            .unwrap_or(sub.len());
        return Some(sub[..end].to_string());
    }
    
    if let Some(pos) = command.find("nix run ") {
        let sub = &command[pos + "nix run ".len()..];
        let end = sub.find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ';')
            .unwrap_or(sub.len());
        let uri = sub[..end].trim();
        if !uri.is_empty() {
            return Some(uri.to_string());
        }
    }
    None
}

/// Evaluates the package version using nix eval.
fn resolve_package_version(uri: &str) -> String {
    let cmd = format!(
        ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix eval --raw {}.version 2>/dev/null",
        uri
    );
    let output = std::process::Command::new("sh")
        .args(&["-c", &cmd])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !ver.is_empty() {
                return ver;
            }
        }
    }

    let cmd_name = format!(
        ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix eval --raw {}.name 2>/dev/null",
        uri
    );
    let output_name = std::process::Command::new("sh")
        .args(&["-c", &cmd_name])
        .output();
        
    if let Ok(out) = output_name {
        if out.status.success() {
            let name_ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some(pos) = name_ver.rfind('-') {
                let ver = &name_ver[pos + 1..];
                if !ver.is_empty() && ver.chars().next().unwrap().is_digit(10) {
                    return ver.to_string();
                }
            }
            if !name_ver.is_empty() {
                return name_ver;
            }
        }
    }
    
    "unknown".to_string()
}

/// Retrieves a cached package version, evaluating it if missing or unknown.
fn get_cached_version(uri: &str) -> String {
    let cache_path = "/boot/config/plugins/nix/.version_cache.json";
    let mut cache: HashMap<String, String> = std::fs::read_to_string(cache_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default();

    if let Some(ver) = cache.get(uri) {
        if ver != "unknown" {
            return ver.clone();
        }
    }

    let ver = resolve_package_version(uri);
    if ver != "unknown" {
        cache.insert(uri.to_string(), ver.clone());
        if let Ok(content) = serde_json::to_string(&cache) {
            let _ = std::fs::write(cache_path, content);
        }
    }
    ver
}

/// Resolves a clickable URL for a package/flake URI (pointing to search.nixos.org or GitHub).
fn get_package_link_url(uri: &str) -> Option<String> {
    if uri.starts_with("nixpkgs#") {
        let short_name = uri.replace("nixpkgs#", "");
        return Some(format!(
            "https://search.nixos.org/packages?channel=unstable&query={}",
            short_name
        ));
    }
    if uri.starts_with("github:") {
        let clean_uri = uri.replace("github:", "");
        let base_path = clean_uri.split('#').next().unwrap_or("");
        let parts: Vec<&str> = base_path.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("https://github.com/{}/{}", parts[0], parts[1]));
        }
    }
    None
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
            let status_lower = s.status.to_lowercase();
            let is_stopped = status_lower == "stopped"
                || status_lower == "completed"
                || status_lower == "terminating";

            let status_subtext = if is_running {
                r#"<span style="color: #2ecc71;">●</span> started"#
            } else if is_stopped && s.exit_code.unwrap_or(0) == 0 {
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

            let cmd = config
                .as_ref()
                .and_then(|c| c.processes.get(&s.name))
                .map(|p| p.command.as_str())
                .unwrap_or("");
            
            let uri = extract_package_uri(cmd).unwrap_or_else(|| format!("nixpkgs#{}", s.name));
            let version = get_cached_version(&uri);

            let version_html = if version != "unknown" {
                if let Some(link_url) = get_package_link_url(&uri) {
                    format!(
                        r#"<div style="display: flex; flex-direction: column; gap: 2px;">
                            <strong><a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a></strong>
                            <span style="color: #2ecc71; font-weight: 500; font-size: 11px;">up-to-date</span>
                        </div>"#,
                        link_url, version
                    )
                } else {
                    format!(
                        r#"<div style="display: flex; flex-direction: column; gap: 2px;">
                            <strong>{}</strong>
                            <span style="color: #2ecc71; font-weight: 500; font-size: 11px;">up-to-date</span>
                        </div>"#,
                        version
                    )
                }
            } else {
                r#"<div style="display: flex; flex-direction: column; gap: 2px;">
                    <strong>-</strong>
                    <span style="color: #2ecc71; font-weight: 500; font-size: 11px;">up-to-date</span>
                </div>"#.to_string()
            };

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
                format!(r#"<button type="button" class="nix-btn" disabled title="Service is running"><i class="fa fa-play"></i></button>"#)
            };

            let stop_btn = if is_running {
                format!(r#"<button type="button" class="nix-btn" onclick="serviceAction('{}', 'stop')" title="Stop"><i class="fa fa-stop"></i></button>"#, s.name)
            } else {
                format!(r#"<button type="button" class="nix-btn" disabled title="Service is stopped"><i class="fa fa-stop"></i></button>"#)
            };

            let edit_btn = format!(
                r#"<button type="button" class="nix-btn" onclick="editService('{}')" title="Edit Config"><i class="fa fa-edit"></i></button>"#,
                s.name
            );

            let logs_btn = format!(r#"<button type="button" class="nix-btn" onclick="openLogs('{}')" title="Logs"><i class="fa fa-file-text-o"></i></button>"#, s.name);

            let remove_btn = format!(
                r#"<button type="button" class="nix-btn" style="color: #e74c3c; border-color: #e74c3c;" onclick="removeService('{}')" title="Remove"><i class="fa fa-trash-o"></i></button>"#,
                s.name
            );

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
                            {}
                            {}
                        </div>
                    </td>
                </tr>"#,
                app_html, version_html, lan_ip_port_html, volume_mappings_html, autostart_html, start_btn, stop_btn, edit_btn, logs_btn, remove_btn
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

            let mut meta_links = Vec::new();
            if let Some(ref lic) = r.license {
                if !lic.trim().is_empty() {
                    meta_links.push(format!(r#"<span><i class="fa fa-certificate" style="margin-right: 3px;"></i>{}</span>"#, lic));
                }
            }
            if let Some(ref hp) = r.homepage {
                if !hp.trim().is_empty() {
                    meta_links.push(format!(r#"<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;"><i class="fa fa-globe" style="margin-right: 3px;"></i>Homepage</a>"#, hp));
                }
            }
            if let Some(ref pos) = r.position {
                if !pos.trim().is_empty() {
                    meta_links.push(format!(r#"<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;"><i class="fa fa-code" style="margin-right: 3px;"></i>Source</a>"#, pos));
                }
            }

            let meta_html = if meta_links.is_empty() {
                "".to_string()
            } else {
                format!(
                    r#"<div style="margin-top: 6px; font-size: 11px; display: flex; gap: 12px; flex-wrap: wrap; align-items: center; color: #888;">{}</div>"#,
                    meta_links.join(r#"<span style="color: #444;">|</span>"#)
                )
            };

            let description_cell = format!("<div>{}</div>{}", r.description, meta_html);

            let short_name = r.package_name.replace("nixpkgs#", "");
            let package_link = format!(
                r#"<a href="https://search.nixos.org/packages?channel=unstable&query={}" target="_blank" style="color: #00a1ff; text-decoration: none;"><code>{}</code> <i class="fa fa-external-link" style="font-size: 10px; margin-left: 2px;"></i></a>"#,
                short_name, short_name
            );

            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                package_link, r.version, description_cell, action_buttons
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_package_link_url() {
        assert_eq!(
            get_package_link_url("nixpkgs#sonarr"),
            Some("https://search.nixos.org/packages?channel=unstable&query=sonarr".to_string())
        );
        assert_eq!(
            get_package_link_url("github:numtide/blueprint#my-service"),
            Some("https://github.com/numtide/blueprint".to_string())
        );
        assert_eq!(
            get_package_link_url("github:numtide/blueprint"),
            Some("https://github.com/numtide/blueprint".to_string())
        );
        assert_eq!(
            get_package_link_url("/path/to/local/flake"),
            None
        );
    }
}

