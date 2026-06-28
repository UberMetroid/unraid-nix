use crate::api::utils::{get_host_ips, get_service_web_port};
use crate::api::utils::{html_escape, js_escape};
use crate::process::{get_services_status, is_supervisor_running, ServiceStatus};

pub(super) fn get_sorted_statuses(api_port: u16) -> Result<Vec<ServiceStatus>, String> {
    let mut statuses = get_services_status(api_port)?;
    statuses.sort_by_key(|a| a.name.to_lowercase());
    Ok(statuses)
}

/// Renders the HTML `<tr>` element for a single service status. Pure
/// formatter — no I/O. Used by both `render_dashboard_rows` (initial
/// render) and `dashboard_diff` (incremental patch).
pub(super) fn render_single_row(s: &ServiceStatus) -> String {
    let host_ips = get_host_ips();
    let ip_str = host_ips
        .first()
        .map(|h| h.ip.as_str())
        .unwrap_or("127.0.0.1");

    let is_running = s.status.to_lowercase() == "running";
    let status_text = if is_running { "Running" } else { "Stopped" };
    let status_color = if is_running { "#2ecc71" } else { "#e74c3c" };
    let shadow = if is_running {
        "0 0 5px #2ecc71"
    } else {
        "none"
    };

    let port = get_service_web_port(&s.name);
    let name_link_html = if let Some(p) = port {
        format!(
            r#"<a href="http://{}:{p}/" target="_blank" title="Open Web UI" style="text-decoration: none; color: inherit; display: inline-flex; align-items: center; gap: 6px;">
                <img src="/plugins/nix/api.php?action=get-icon&service={}" style="width: 16px; height: 16px; border-radius: 2px; vertical-align: middle;" />
                <span style="vertical-align: middle; font-weight: 500;">{}</span>
                <i class="fa fa-external-link" style="font-size: 8px; color: #00a1ff; opacity: 0.7; vertical-align: middle;"></i>
            </a>"#,
            html_escape(ip_str),
            html_escape(&s.name),
            html_escape(&s.name)
        )
    } else {
        format!(
            r#"<span style="display: inline-flex; align-items: center; gap: 6px;">
                <img src="/plugins/nix/api.php?action=get-icon&service={}" style="width: 16px; height: 16px; border-radius: 2px; vertical-align: middle;" />
                <span style="vertical-align: middle; font-weight: 500;">{}</span>
            </span>"#,
            html_escape(&s.name),
            html_escape(&s.name)
        )
    };

    let btn_icon = if is_running { "fa-stop" } else { "fa-play" };
    let btn_action = if is_running { "stop" } else { "start" };
    let btn_title = if is_running {
        "Stop Service"
    } else {
        "Start Service"
    };

    let gpu_indicator = if s.gpu_active.unwrap_or(false) {
        r#"<i class="fa fa-microchip nix-dash-gpu-active" style="font-size: 11px; color: #00a1ff; vertical-align: middle;" title="GPU Active"></i>"#
    } else {
        r#"<span style="color: #666;">-</span>"#
    };

    format!(
        r#"<tr data-service="{}" style="border-bottom: 1px solid rgba(255, 255, 255, 0.03);">
            <td style="padding: 8px; vertical-align: middle;">
                {}
            </td>
            <td style="padding: 8px; vertical-align: middle;">
                <span class="status-dot" style="background: {}; display: inline-block; width: 6px; height: 6px; border-radius: 50%; margin-right: 6px; box-shadow: {}; vertical-align: middle;"></span>
                <span class="status-text" style="font-size: 11px; vertical-align: middle;">{}</span>
            </td>
            <td style="padding: 8px; vertical-align: middle; text-align: center;" class="gpu-wrapper">
                {}
            </td>
            <td style="padding: 8px; vertical-align: middle; text-align: right;">
                <button type="button" class="nix-dash-toggle-btn" onclick="toggleDashService('{}', '{}')" title="{}" style="background: none; border: none; padding: 4px; cursor: pointer; color: #a0a0a5; outline: none; display: inline-flex; align-items: center; justify-content: center;">
                    <i class="fa {}" style="font-size: 10px; transition: color 0.15s ease;"></i>
                </button>
            </td>
        </tr>"#,
        html_escape(&s.name),
        name_link_html,
        status_color,
        shadow,
        status_text,
        gpu_indicator,
        html_escape(&js_escape(&s.name)),
        btn_action,
        btn_title,
        btn_icon
    )
}

/// Renders ONLY the service rows as HTML (used for initial widget render).
pub fn render_dashboard_rows(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<tr><td colspan="4" style="padding: 8px; text-align: center; color: #999;">Supervisor not running</td></tr>"#.to_string();
    }

    let statuses = match get_sorted_statuses(api_port) {
        Ok(s) => s,
        Err(_) => return r#"<tr><td colspan="4" style="padding: 8px; text-align: center; color: #e74c3c;">Error reading statuses</td></tr>"#.to_string(),
    };

    if statuses.is_empty() {
        return r#"<tr><td colspan="4" style="padding: 8px; text-align: center; color: #999;">No configured services</td></tr>"#.to_string();
    }

    statuses.iter().map(render_single_row).collect()
}

/// Returns the service statuses directly as a JSON string (used for dynamic updates).
pub fn render_dashboard_json(api_port: u16) -> String {
    get_sorted_statuses(api_port)
        .map(|statuses| serde_json::to_string(&statuses).unwrap_or_else(|_| "[]".to_string()))
        .unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_status(name: &str, status: &str, cpu: Option<f32>) -> ServiceStatus {
        ServiceStatus {
            name: name.to_string(),
            status: status.to_string(),
            pid: Some(1),
            cpu,
            memory: Some(1024),
            uptime_nanoseconds: Some(60_000_000_000),
            exit_code: None,
            gpu_active: Some(false),
            io_read: Some(0),
            io_write: Some(0),
            gpu_stats: None,
        }
    }

    #[test]
    fn test_render_single_row_running_status_has_green_dot_and_stop_action() {
        let s = make_status("radarr", "Running", Some(2.5));
        let html = render_single_row(&s);
        assert!(
            html.contains("data-service=\"radarr\""),
            "row must carry data-service attr"
        );
        assert!(
            html.contains("status-dot"),
            "row must include status-dot span"
        );
        assert!(html.contains("#2ecc71"), "running row must use green color");
        assert!(html.contains("'stop'"), "running row must wire stop action");
        assert!(
            html.contains("fa-stop"),
            "running row must use fa-stop icon"
        );
        assert!(
            html.contains(">Running<"),
            "running row must show Running text"
        );
    }

    #[test]
    fn test_render_single_row_stopped_status_has_red_dot_and_start_action() {
        let s = make_status("sonarr", "Stopped", None);
        let html = render_single_row(&s);
        assert!(html.contains("#e74c3c"), "stopped row must use red color");
        assert!(
            html.contains("'start'"),
            "stopped row must wire start action"
        );
        assert!(
            html.contains("fa-play"),
            "stopped row must use fa-play icon"
        );
        assert!(
            html.contains(">Stopped<"),
            "stopped row must show Stopped text"
        );
    }
}
