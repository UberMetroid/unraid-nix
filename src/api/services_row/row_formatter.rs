use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::{HostAddr, get_service_web_port, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};

use super::cells::{
    render_lan_ip_port_cell,
    render_resources_cell, render_autostart_cell,
};

pub fn render_service_row(
    s: &ServiceStatus,
    config: &Option<ProcessComposeConfig>,
    host_ips: &[HostAddr],
) -> String {
    let is_running = s.status.to_lowercase() == "running";
    let status_lower = s.status.to_lowercase();
    let is_stopped = status_lower == "stopped"
        || status_lower == "completed"
        || status_lower == "terminating";

    let status_class = if is_running {
        "status-running"
    } else if is_stopped && s.exit_code.unwrap_or(0) == 0 {
        "status-stopped"
    } else {
        "status-failed"
    };

    let status_label = if is_running {
        "RUNNING"
    } else if is_stopped && s.exit_code.unwrap_or(0) == 0 {
        "STOPPED"
    } else {
        "FAILED"
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
                r#"v<a href="{}" target="_blank" style="color: var(--nix-accent); text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 8px;"></i></a>"#,
                link_url, version
            )
        } else {
            format!("v{}", version)
        }
    } else {
        "v0.0.0".to_string()
    };

    let port_num = get_service_web_port(&s.name);
    let metadata_file = format!("/boot/config/plugins/nix/metadata/{}.json", s.name);
    let mut bind_address_override = None;
    let mut appdata_path = None;
    let mut extra_binds_vec = Vec::new();
    let mut gpus_override = None;
    let mut legacy_gpu = None;

    if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            appdata_path = meta.get("appdata")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

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

    let lan_ip_port_html = render_lan_ip_port_cell(port_num, is_running, host_ips, &bind_address_override);

    let autostart_enabled = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .and_then(|p| p.availability.as_ref())
        .map(|a| a.restart.to_lowercase() == "always")
        .unwrap_or(true);

    let autostart_html = render_autostart_cell(&s.name, autostart_enabled);

    let start_btn = if !is_running {
        format!(r#"<button type="button" class="nix-btn nix-btn-sm" onclick="serviceAction('{}', 'start')" title="Start"><i class="fa fa-play" style="color: #2ecc71;"></i></button>"#, s.name)
    } else {
        format!(r#"<button type="button" class="nix-btn nix-btn-sm" disabled title="Service is running"><i class="fa fa-play" style="color: var(--nix-text-muted);"></i></button>"#)
    };

    let stop_btn = if is_running {
        format!(r#"<button type="button" class="nix-btn nix-btn-sm" onclick="serviceAction('{}', 'stop')" title="Stop"><i class="fa fa-stop" style="color: #e74c3c;"></i></button>"#, s.name)
    } else {
        format!(r#"<button type="button" class="nix-btn nix-btn-sm" disabled title="Service is stopped"><i class="fa fa-stop" style="color: var(--nix-text-muted);"></i></button>"#)
    };

    let edit_btn = format!(
        r#"<button type="button" class="nix-btn nix-btn-sm" onclick="editService('{}')" title="Edit Config"><i class="fa fa-edit"></i></button>"#,
        s.name
    );

    let logs_btn = format!(r#"<button type="button" class="nix-btn nix-btn-sm" onclick="openLogs('{}')" title="Logs"><i class="fa fa-file-text-o"></i></button>"#, s.name);

    let mut mapped_drives = Vec::new();
    if let Some(ref path) = appdata_path {
        if !path.trim().is_empty() {
            mapped_drives.push((path.clone(), "/config".to_string()));
        }
    }
    for (host, sandbox) in extra_binds_vec {
        mapped_drives.push((host, sandbox));
    }

    let mapped_drives_html = if mapped_drives.is_empty() {
        r#"<span style="color: var(--nix-text-muted);">None</span>"#.to_string()
    } else {
        let mut lines = Vec::new();
        for (h, s) in &mapped_drives {
            let h_short = if h.len() > 40 { format!("...{}", &h[h.len()-37..]) } else { h.clone() };
            let s_short = if s.len() > 30 { format!("...{}", &s[s.len()-27..]) } else { s.clone() };
            lines.push(format!(
                r#"<div style="font-family: monospace; font-size: 10px; color: var(--nix-text-primary); text-overflow: ellipsis; white-space: nowrap; overflow: hidden;" title="{} → {}">{} → {}</div>"#,
                h, s, h_short, s_short
            ));
        }
        lines.join("")
    };

    let ports_list = get_service_ports(&s.name);
    let ports_html = if ports_list.is_empty() {
        r#"<span style="color: var(--nix-text-muted);">None</span>"#.to_string()
    } else {
        let mut lines = Vec::new();
        for p in &ports_list {
            lines.push(format!(
                r#"<div style="font-family: monospace; font-size: 10px; color: var(--nix-text-primary);">{} → {} (TCP)</div>"#,
                p.host, p.container
            ));
        }
        lines.join("")
    };

    let rollback_html = format!(
        r#"<div style="display: flex; align-items: center; justify-content: space-between; width: 100%; height: 16px;">
            <span style="color: var(--nix-text-primary); font-family: monospace; font-size: 10px;">Gen 1 (Active)</span>
            <button type="button" class="nix-btn" style="padding: 1px 4px; font-size: 8px; line-height: 1; min-width: unset; margin: 0; height: 16px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-tertiary); color: var(--nix-text-secondary); border-radius: 3px; cursor: pointer;" onclick="alert('Rollback support is coming in a future update!')">Rollback</button>
        </div>"#
    );

    let time_html = if is_running {
        format!(
            r#"<span style="font-size: 10px; color: var(--nix-text-secondary); font-family: monospace; line-height: 1;">{}</span>"#,
            s.uptime()
        )
    } else {
        "".to_string()
    };

    let resources_html = render_resources_cell(&s.name, is_running, s.cpu, s.memory, &gpus_override, &legacy_gpu, &s.gpu_stats);

    use super::static_config::get_service_fa_config;
    let cfg = get_service_fa_config(&s.name);

    format!(
        r#"<div class="nix-preset-card nix-service-card" data-name="{}" style="background: var(--nix-bg-secondary); border: 1px solid var(--nix-border-primary); border-radius: 6px; padding: 16px; display: flex; flex-direction: column; justify-content: space-between; transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease; min-height: 350px; height: auto; position: relative;">
            <div>
                <!-- Top Row: Icon, Name + Path/Version on Left, Uptime & Status Dot on Right -->
                <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; margin-bottom: 10px;">
                    <div style="display: flex; align-items: flex-start; gap: 10px; min-width: 0; flex: 1;">
                        <div style="width: 32px; height: 32px; border-radius: 4px; background: {}; border: 1px solid {}; display: flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0; margin-top: 2px;">
                            <i class="fa {}" style="font-size: 15px;"></i>
                        </div>
                        <div style="display: flex; flex-direction: column; overflow: hidden; min-width: 0; flex: 1;">
                            <strong style="font-size: 14px; color: var(--nix-text-primary); word-break: break-word; overflow-wrap: break-word;" title="{}">{}</strong>
                            <span style="font-family: monospace; color: var(--nix-text-secondary); font-size: 10px; margin-top: 2px; display: inline-flex; align-items: center; gap: 6px; flex-wrap: wrap;">
                                <span>nixpkgs#{}</span>
                                {}
                            </span>
                        </div>
                    </div>
                    <div style="display: flex; align-items: center; gap: 6px; flex-shrink: 0; margin-top: 6px;">
                        {}
                        <span class="status-dot {}" data-service="{}" title="{}" style="margin-top: 0;"></span>
                    </div>
                </div>

                <!-- Info list -->
                <div style="display: flex; flex-direction: column; gap: 8px; font-size: 11px; border-top: 1px solid var(--nix-border-primary); padding-top: 10px;">
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">ACTIVITY</span>
                        <div style="padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">CONNECTION</span>
                        <div style="padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">MOUNTS</span>
                        <div style="display: flex; flex-direction: column; gap: 3px; padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">PORTS</span>
                        <div style="display: flex; flex-direction: column; gap: 3px; padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">ROLLBACK</span>
                        <div style="padding-left: 6px;">{}</div>
                    </div>
                </div>
            </div>

            <!-- Bottom Row: Controls Toolbar -->
            <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--nix-border-primary); padding-top: 10px; margin-top: 12px;">
                <div style="display: flex; gap: 6px; align-items: center;">
                    {}
                    {}
                    {}
                    {}
                    {}
                </div>
                <button type="button" class="nix-btn nix-btn-sm" style="color: #e74c3c; border-color: var(--nix-border-primary); margin: 0; display: inline-flex; align-items: center; justify-content: center; height: 32px; width: 32px;" onclick="removeService('{}')" title="Remove"><i class="fa fa-trash-o" style="color: #e74c3c;"></i></button>
            </div>
        </div>"#,
        s.name, cfg.bg, cfg.border, cfg.color, cfg.icon, s.name, s.name, s.name, version_badge, time_html, status_class, s.name, status_label, resources_html, lan_ip_port_html, mapped_drives_html, ports_html, rollback_html, start_btn, stop_btn, edit_btn, logs_btn, autostart_html, s.name
    )
}

fn get_service_ports(name: &str) -> Vec<crate::sandbox::PortMapping> {
    let mut ports = Vec::new();
    let metadata_path = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(port_val) = val.get("port") {
                    if let Some(num) = port_val.as_u64() {
                        if num > 0 {
                            ports.push(crate::sandbox::PortMapping { host: num as u16, container: num as u16 });
                        }
                    }
                    if let Some(s) = port_val.as_str() {
                        let mappings = crate::sandbox::parse_ports(s);
                        if !mappings.is_empty() {
                            return mappings;
                        }
                    }
                }
            }
        }
    }

    let name_lower = name.to_lowercase();
    let preset_path = crate::config::get_preset_path(&name_lower);
    if std::path::Path::new(&preset_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&preset_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ports_arr) = json.get("default_ports").and_then(|p| p.as_array()) {
                    for port_item in ports_arr {
                        if let (Some(h), Some(c)) = (port_item.get("host").and_then(|v| v.as_u64()), port_item.get("container").and_then(|v| v.as_u64())) {
                            ports.push(crate::sandbox::PortMapping { host: h as u16, container: c as u16 });
                        }
                    }
                }
            }
        }
    }

    if ports.is_empty() {
        if name_lower.contains("sonarr") {
            ports.push(crate::sandbox::PortMapping { host: 8989, container: 8989 });
        } else if name_lower.contains("radarr") {
            ports.push(crate::sandbox::PortMapping { host: 7878, container: 7878 });
        } else if name_lower.contains("jellyfin") {
            ports.push(crate::sandbox::PortMapping { host: 8096, container: 8096 });
        } else if name_lower.contains("syncthing") {
            ports.push(crate::sandbox::PortMapping { host: 8384, container: 8384 });
        }
    }

    ports
}
