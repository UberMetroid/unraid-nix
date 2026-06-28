use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::HostAddr;
use crate::api::utils::{html_escape, js_escape};

use super::data::{
    autodetect_service_port, autostart_enabled_for, collect_mapped_drives, derive_uri,
    extract_metadata, get_service_cmd, status_fields, truncate_path_ellipsis,
    version_badge_for,
};
use super::ports::get_service_ports;
use super::templates;

/// All the data we need to render one service row.
pub(crate) struct RowData {
    pub(crate) status_class: &'static str,
    pub(crate) status_label: &'static str,
    pub(crate) version_badge: String,
    pub(crate) time_html: String,
    pub(crate) lan_ip_port_html: String,
    pub(crate) autostart_html: String,
    pub(crate) mapped_drives_html: String,
    pub(crate) ports_html: String,
    pub(crate) rollback_html: String,
    pub(crate) start_btn: String,
    pub(crate) stop_btn: String,
    pub(crate) edit_btn: String,
    pub(crate) logs_btn: String,
    pub(crate) resources_html: String,
    pub(crate) bg: &'static str,
    pub(crate) border: &'static str,
    pub(crate) color: &'static str,
    pub(crate) icon: String,
    pub(crate) name: String,
}

fn time_html_for(s: &ServiceStatus) -> String {
    if s.status.to_lowercase() == "running" {
        format!(
            r#"<span style="font-size: 10px; color: var(--nix-text-secondary); font-family: monospace; line-height: 1;">{}</span>"#,
            html_escape(&s.uptime())
        )
    } else {
        String::new()
    }
}

fn render_button(label_class: &str, enabled: bool, action: &str, title: &str, fa_icon: &str, color: &str) -> String {
    if enabled {
        format!(
            r#"<button type="button" class="nix-btn nix-btn-sm" onclick="serviceAction('{}', '{}')" title="{}"><i class="fa {}" style="color: {};"></i></button>"#,
            html_escape(&js_escape(label_class)), action, title, fa_icon, color
        )
    } else {
        format!(
            r#"<button type="button" class="nix-btn nix-btn-sm" disabled title="{}"><i class="fa {}" style="color: var(--nix-text-muted);"></i></button>"#,
            title, fa_icon
        )
    }
}

fn mapped_drives_html(drives: &[(String, String)]) -> String {
    if drives.is_empty() {
        return r#"<span style="color: var(--nix-text-muted);">None</span>"#.to_string();
    }
    let mut lines = Vec::new();
    for (h, s) in drives {
        let h_short = truncate_path_ellipsis(h, 40, 37);
        let s_short = truncate_path_ellipsis(s, 30, 27);
        lines.push(format!(
            r#"<div style="font-family: monospace; font-size: 10px; color: var(--nix-text-primary); text-overflow: ellipsis; white-space: nowrap; overflow: hidden;" title="{} → {}">{} → {}</div>"#,
            html_escape(h), html_escape(s), html_escape(&h_short), html_escape(&s_short)
        ));
    }
    lines.join("")
}

fn ports_html(name: &str) -> String {
    let ports_list = get_service_ports(name);
    if ports_list.is_empty() {
        return r#"<span style="color: var(--nix-text-muted);">None</span>"#.to_string();
    }
    let mut lines = Vec::new();
    for p in &ports_list {
        lines.push(format!(
            r#"<div style="font-family: monospace; font-size: 10px; color: var(--nix-text-primary);">{} → {} (TCP)</div>"#,
            p.host, p.container
        ));
    }
    lines.join("")
}

const ROLLBACK_HTML: &str = r#"<div style="display: flex; align-items: center; justify-content: space-between; width: 100%; height: 16px;">
              <span style="color: var(--nix-text-primary); font-family: monospace; font-size: 10px;">Gen 1 (Active)</span>
              <button type="button" class="nix-btn" style="padding: 1px 4px; font-size: 8px; line-height: 1; min-width: unset; margin: 0; height: 16px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-tertiary); color: var(--nix-text-secondary); border-radius: 3px; cursor: pointer;" onclick="alert('Rollback support is coming in a future update!')">Rollback</button>
          </div>"#;

fn build_row_data(s: &ServiceStatus, config: &Option<ProcessComposeConfig>, host_ips: &[HostAddr]) -> RowData {
    let (status_class, status_label) = status_fields(&s.status);
    let is_running = status_class == "status-running";

    let cmd = get_service_cmd(config, &s.name);
    let uri = derive_uri(cmd.as_deref().unwrap_or(""), &s.name);
    let version_badge = version_badge_for(&uri);

    let port_num = autodetect_service_port(&s.name);
    let meta = extract_metadata(&s.name);
    let autostart_on = autostart_enabled_for(config, &s.name);
    let mapped_drives = collect_mapped_drives(&meta);

    use crate::api::services_row::cells::{
        render_autostart_cell, render_lan_ip_port_cell, render_resources_cell,
    };
    use crate::api::services_row::static_config::get_service_fa_config;

    let lan_ip_port_html = render_lan_ip_port_cell(port_num, is_running, host_ips, &meta.bind_address_override);
    let autostart_html = render_autostart_cell(&s.name, autostart_on);
    let resources_html = render_resources_cell(
        &s.name,
        is_running,
        s.cpu,
        s.memory,
        &meta.gpus_override,
        &meta.legacy_gpu,
        &s.gpu_stats,
    );

    let start_btn = render_button(&s.name, !is_running, "start", "Start", "fa-play", "#2ecc71");
    let stop_btn = render_button(&s.name, is_running, "stop", "Stop", "fa-stop", "#e74c3c");
    let edit_btn = format!(
        r#"<button type="button" class="nix-btn nix-btn-sm" onclick="editService('{}')" title="Edit Config"><i class="fa fa-edit"></i></button>"#,
        html_escape(&js_escape(&s.name))
    );
    let logs_btn = format!(
        r#"<button type="button" class="nix-btn nix-btn-sm" onclick="openLogs('{}')" title="Logs"><i class="fa fa-file-text-o"></i></button>"#,
        html_escape(&js_escape(&s.name))
    );

    let mapped_drives_html = mapped_drives_html(&mapped_drives);
    let ports_html = ports_html(&s.name);
    let time_html = time_html_for(s);

    let cfg = get_service_fa_config(&s.name);

    RowData {
        status_class,
        status_label,
        version_badge,
        time_html,
        lan_ip_port_html,
        autostart_html,
        mapped_drives_html,
        ports_html,
        rollback_html: ROLLBACK_HTML.to_string(),
        start_btn,
        stop_btn,
        edit_btn,
        logs_btn,
        resources_html,
        bg: cfg.bg,
        border: cfg.border,
        color: cfg.color,
        icon: cfg.icon,
        name: s.name.clone(),
    }
}

/// Renders a single service status as a full HTML row for the services
/// dashboard table. Delegates data extraction to `data::*` helpers and
/// HTML composition to `templates::build_row_html`.
pub fn render_service_row(
    s: &ServiceStatus,
    config: &Option<ProcessComposeConfig>,
    host_ips: &[HostAddr],
) -> String {
    let d = build_row_data(s, config, host_ips);

    templates::build_row_html(&templates::RowTemplateData {
        name: &d.name,
        bg: d.bg,
        border: d.border,
        color: d.color,
        icon: &d.icon,
        version_badge: &d.version_badge,
        time_html: &d.time_html,
        status_class: d.status_class,
        status_label: d.status_label,
        resources_html: &d.resources_html,
        lan_ip_port_html: &d.lan_ip_port_html,
        mapped_drives_html: &d.mapped_drives_html,
        ports_html: &d.ports_html,
        rollback_html: &d.rollback_html,
        start_btn: &d.start_btn,
        stop_btn: &d.stop_btn,
        edit_btn: &d.edit_btn,
        logs_btn: &d.logs_btn,
        autostart_html: &d.autostart_html,
    })
}