use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::{HostAddr, get_service_web_port, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};

use super::cells::{
    render_app_cell, render_lan_ip_port_cell,
    render_resources_cell, render_gpu_cell, render_autostart_cell,
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

    let app_html = render_app_cell(&s.name, &version_badge, status_subtext);

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

    let resources_html = render_resources_cell(&s.name, is_running, s.cpu, s.memory);
    let gpu_html = render_gpu_cell(&s.name, &gpus_override, &legacy_gpu, &s.gpu_stats);

    format!(
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
                    {}
                    {}
                </div>
            </td>
            <td>
                <div style="display: inline-block; vertical-align: middle;">{}</div>
            </td>
        </tr>"#,
        app_html, lan_ip_port_html, resources_html, gpu_html, start_btn, stop_btn, edit_btn, logs_btn, remove_btn, autostart_html
    )
}
