use crate::process::{get_services_status, is_supervisor_running};

/// Renders a dashboard widget summary showing all active Nix services.
pub fn render_dashboard_widget(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<tr><td colspan="3" class="text-center text-muted">Nix supervisor not running.</td></tr>"#.to_string();
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
