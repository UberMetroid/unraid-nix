use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::{HostAddr, get_service_web_port, get_service_appdata_path, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};

fn get_service_fa_icon(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();
    if name_lower.contains("jellyfin") || name_lower.contains("plex") || name_lower.contains("emby") {
        "fa-play-circle"
    } else if name_lower.contains("sonarr") || name_lower.contains("sickrage") {
        "fa-television"
    } else if name_lower.contains("radarr") || name_lower.contains("couchpotato") {
        "fa-film"
    } else if name_lower.contains("prowlarr") || name_lower.contains("jackett") {
        "fa-search"
    } else if name_lower.contains("transmission") || name_lower.contains("sabnzbd") || name_lower.contains("nzbget") || name_lower.contains("qbittorrent") || name_lower.contains("deluge") {
        "fa-download"
    } else if name_lower.contains("syncthing") || name_lower.contains("nextcloud") || name_lower.contains("owncloud") {
        "fa-refresh"
    } else if name_lower.contains("home-assistant") || name_lower.contains("homeassistant") || name_lower.contains("hass") {
        "fa-home"
    } else if name_lower.contains("jellyseerr") || name_lower.contains("overseerr") {
        "fa-eye"
    } else {
        "fa-server"
    }
}

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

    let cmd = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .map(|p| p.command.as_str())
        .unwrap_or("");
    
    let uri = extract_package_uri(cmd).unwrap_or_else(|| format!("nixpkgs#{}", s.name));
    let version = get_cached_version(&uri);

    let version_badge = if version != "unknown" {
        if let Some(link_url) = get_package_link_url(&uri) {
            format!(
                r#"<div style="font-size: 11px; color: #a0a0a5; margin-top: -1px; margin-bottom: -1px;">v<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 8px;"></i></a> <span style="color: #2ecc71; font-weight: 500;">(up-to-date)</span></div>"#,
                link_url, version
            )
        } else {
            format!(
                r#"<div style="font-size: 11px; color: #a0a0a5; margin-top: -1px; margin-bottom: -1px;">v{} <span style="color: #2ecc71; font-weight: 500;">(up-to-date)</span></div>"#,
                version
            )
        }
    } else {
        "".to_string()
    };

    let fa_icon = get_service_fa_icon(&s.name);
    let app_html = format!(
        r#"<div style="display: flex; align-items: center; gap: 10px;">
            <div style="width: 28px; height: 28px; border-radius: 4px; background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.2); display: inline-flex; align-items: center; justify-content: center; color: #00a1ff; flex-shrink: 0;">
                <i class="fa {}" style="font-size: 14px;"></i>
            </div>
            <div style="display: flex; flex-direction: column; gap: 2px;">
                <strong style="font-size: 13px;">{}</strong>
                {}
                <div style="font-size: 11px; color: #a0a0a5;">{}</div>
            </div>
        </div>"#,
        fa_icon, s.name, version_badge, status_subtext
    );

    let port_num = get_service_web_port(&s.name);
    let metadata_file = format!("/boot/config/plugins/nix/metadata/{}.json", s.name);
    let mut bind_address_override = None;
    let mut extra_binds_vec = Vec::new();
    let mut gpus_override = None;
    let mut legacy_gpu = None;

    if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            bind_address_override = meta.get("bind_address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
                
            gpus_override = meta.get("gpus")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            legacy_gpu = meta.get("gpu")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
                
            if let Some(binds_val) = meta.get("extra_binds") {
                if let Some(binds_str) = binds_val.as_str() {
                    if let Ok(parsed_binds) = serde_json::from_str::<serde_json::Value>(binds_str) {
                        if let Some(arr) = parsed_binds.as_array() {
                            for item in arr {
                                if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|h| h.as_str()), item.get("sandbox").and_then(|s| s.as_str())) {
                                    extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                                }
                            }
                        }
                    }
                } else if let Some(arr) = binds_val.as_array() {
                    for item in arr {
                        if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|h| h.as_str()), item.get("sandbox").and_then(|s| s.as_str())) {
                            extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                        }
                    }
                }
            }
        }
    }

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

    let mut volume_mappings = Vec::new();
    if home_path != "-" && !home_path.is_empty() {
        volume_mappings.push(format!(
            r#"<div style="margin-bottom: 4px;"><span style="color: #a0a0a5; font-family: monospace;">/config</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code></div>"#,
            home_path
        ));
    }
    for (host, sandbox) in extra_binds_vec {
        if !host.is_empty() && !sandbox.is_empty() {
            volume_mappings.push(format!(
                r#"<div style="margin-bottom: 4px;"><span style="color: #a0a0a5; font-family: monospace;">{}</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code></div>"#,
                sandbox, host
            ));
        }
    }

    let volume_mappings_html = if volume_mappings.is_empty() {
        "-".to_string()
    } else {
        volume_mappings.join("")
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

    let gpus_display = match gpus_override {
        Some(ref g) if !g.trim().is_empty() => {
            let mut badges = Vec::new();
            for part in g.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    let display_part = if p.starts_with("nvidia-") {
                        p.replace("nvidia-", "GPU-")
                    } else {
                        p.to_string()
                    };
                    badges.push(format!(
                        r#"<div style="margin-bottom: 4px;"><span style="background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.25); border-radius: 3px; padding: 2px 6px; font-size: 10px; color: #00a1ff; font-family: monospace; display: inline-block;">{}</span></div>"#,
                        display_part
                    ));
                }
            }
            if badges.is_empty() {
                r#"<span style="color: #777;">-</span>"#.to_string()
            } else {
                badges.join("")
            }
        }
        _ => {
            if let Some(ref lg) = legacy_gpu {
                if lg == "1" || lg == "true" {
                    r#"<div style="margin-bottom: 4px;"><span style="background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.25); border-radius: 3px; padding: 2px 6px; font-size: 10px; color: #00a1ff; font-family: monospace; display: inline-block;">All GPUs</span></div>"#.to_string()
                } else {
                    r#"<span style="color: #777;">-</span>"#.to_string()
                }
            } else {
                r#"<span style="color: #777;">-</span>"#.to_string()
            }
        }
    };

    let resources_html = {
        let mut res = String::new();
        if is_running {
            let cpu_str = if let Some(cpu) = s.cpu {
                format!("{:.1}% CPU", cpu)
            } else {
                "0.0% CPU".to_string()
            };
            let mem_str = if let Some(mem) = s.memory {
                let mb = mem as f64 / 1_048_576.0;
                format!("{:.1} MB RAM", mb)
            } else {
                "0.0 MB RAM".to_string()
            };
            res.push_str(&format!(
                r#"<div style="font-size: 11px; color: #eee; font-family: monospace; line-height: 1.4;">{}</div>
                   <div style="font-size: 11px; color: #a0a0a5; font-family: monospace; line-height: 1.4; margin-bottom: 4px;">{}</div>"#,
                cpu_str, mem_str
            ));
        }
        if gpus_display != r#"<span style="color: #777;">-</span>"# {
            res.push_str(&gpus_display);
        } else if !is_running {
            res.push_str(r#"<span style="color: #777;">-</span>"#);
        }
        res
    };

    format!(
        r#"<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <div style="display: flex; align-items: center; gap: 15px;">
                    <div>{}</div>
                    <div class="nix-actions-wrapper">
                        {}
                        {}
                        {}
                        {}
                        {}
                    </div>
                </div>
            </td>
        </tr>"#,
        app_html, lan_ip_port_html, volume_mappings_html, resources_html, autostart_html, start_btn, stop_btn, edit_btn, logs_btn, remove_btn
    )
}
