use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::{HostAddr, get_service_web_port, get_service_appdata_path, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};

pub fn render_service_row(s: &ServiceStatus, config: &Option<ProcessComposeConfig>, host_ips: &[HostAddr]) -> String {
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
    let metadata_file = format!("/boot/config/plugins/nix/metadata/{}.json", s.name);
    let bind_address_override = if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            meta.get("bind_address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let lan_ip_port_html = if let Some(port) = port_num {
        let mut ip_links = Vec::new();
        let has_specific_bind = if let Some(ref addr) = bind_address_override {
            let a = addr.trim();
            !a.is_empty() && a != "0.0.0.0" && a != "*"
        } else {
            false
        };

        for addr in host_ips {
            if has_specific_bind {
                if let Some(ref target) = bind_address_override {
                    if addr.ip != target.trim() {
                        continue;
                    }
                }
            }

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

        if ip_links.is_empty() && has_specific_bind {
            if let Some(ref target) = bind_address_override {
                let link = if is_running {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a> <span style="font-size: 10px; color: #777; font-family: monospace;">(override)</span></div>"##,
                        target, port, target, port
                    )
                } else {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><span style="color: #888;">{}:{}</span> <span style="font-size: 10px; color: #555; font-family: monospace;">(override)</span></div>"##,
                        target, port
                    )
                };
                ip_links.push(link);
            }
        }

        if ip_links.is_empty() {
            "-".to_string()
        } else {
            ip_links.join("")
        }
    } else {
        "-".to_string()
    };

    let home_path = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .map(|p| get_service_appdata_path(&s.name, &p.command))
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

    format!(
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
    )
}
