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

    let resources_html = render_resources_cell(&s.name, is_running, s.cpu, s.memory, &gpus_override, &legacy_gpu, &s.gpu_stats);

    use super::static_config::get_service_fa_config;
    let cfg = get_service_fa_config(&s.name);

    format!(
        r#"<div class="nix-preset-card nix-service-card" data-name="{}" style="background: var(--nix-bg-secondary); border: 1px solid var(--nix-border-primary); border-radius: 6px; padding: 16px; display: flex; flex-direction: column; justify-content: space-between; transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease; height: 180px; position: relative;">
            <div>
                <!-- Top Row: Icon and Title info on Left, Control elements on Right -->
                <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 10px;">
                    <div style="display: flex; align-items: center; gap: 10px; min-width: 0; flex: 1;">
                        <div style="width: 32px; height: 32px; border-radius: 4px; background: {}; border: 1px solid {}; display: flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0;">
                            <i class="fa {}" style="font-size: 15px;"></i>
                        </div>
                        <div style="display: flex; flex-direction: column; overflow: hidden;">
                            <strong style="font-size: 14px; color: var(--nix-text-primary); text-overflow: ellipsis; white-space: nowrap; overflow: hidden;" title="{}">{}</strong>
                            <span style="font-family: monospace; color: var(--nix-text-secondary); font-size: 10px; margin-top: 2px;">nixpkgs#{}</span>
                        </div>
                    </div>
                    
                    <div style="display: flex; flex-direction: column; align-items: flex-end; gap: 4px; flex-shrink: 0;">
                        <div style="display: inline-flex; align-items: center; gap: 4px;" title="Auto Restart">
                            <span style="font-size: 8px; color: var(--nix-text-muted); font-weight: 600;">AUTOSTART</span>
                            {}
                        </div>
                        <div class="nix-service-status-badge" data-service="{}">
                            <span class="status-indicator {}">{}</span>
                        </div>
                        <div style="display: flex; gap: 3px; margin-top: 2px;">
                            {}
                            {}
                            {}
                            {}
                            <button type="button" class="nix-btn nix-btn-sm" style="color: #e74c3c; border-color: var(--nix-border-primary); margin: 0;" onclick="removeService('{}')" title="Remove"><i class="fa fa-trash-o" style="color: #e74c3c;"></i></button>
                        </div>
                    </div>
                </div>

                <!-- Info list -->
                <div style="display: flex; flex-direction: column; gap: 6px; font-size: 11px; margin-top: 10px; border-top: 1px solid var(--nix-border-primary); padding-top: 8px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary);">Version / Info:</span>
                        <span style="text-align: right; color: var(--nix-text-primary);">{}</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary);">Web Interface:</span>
                        <span style="text-align: right; max-width: 170px; word-break: break-all; overflow-wrap: break-word;">{}</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: flex-start; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); margin-top: 2px;">Resources:</span>
                        <div style="text-align: right; display: flex; flex-direction: column; align-items: flex-end;">{}</div>
                    </div>
                </div>
            </div>
        </div>"#,
        s.name, cfg.bg, cfg.border, cfg.color, cfg.icon, s.name, s.name, s.name, autostart_html, s.name, status_class, status_label, start_btn, stop_btn, edit_btn, logs_btn, s.name, version_badge, lan_ip_port_html, resources_html
    )
}
