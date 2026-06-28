use crate::process::{get_services_status, is_supervisor_running};
use crate::api::utils::get_host_ips;

/// Renders the services dashboard table as an HTML string.
/// Mirrors the styling and visual cues of Unraid's native Docker container list.
pub fn render_services_table(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<div class="alert alert-warning"><i class="fa fa-exclamation-triangle"></i> Nix process supervisor (process-compose) is not running. Start the array to launch the services.</div>"#.to_string();
    }

    let mut statuses = match get_services_status(api_port) {
        Ok(s) => s,
        Err(e) => return format!(r#"<div class="alert alert-danger"><i class="fa fa-times"></i> Error connecting to supervisor API: {}</div>"#, e),
    };

    statuses.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let config_path = "/boot/config/plugins/nix/process-compose.yml";
    let config = crate::config::load_config(config_path).ok();
    let host_ips = get_host_ips();

    let mut html = r#"<div class="nix-presets-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; margin-bottom: 20px;">"#.to_string();

    if statuses.is_empty() {
        html.push_str(r#"<div style="grid-column: 1 / -1; text-align: center; color: var(--nix-text-muted); padding: 45px 0;">No Nix Flake services configured. Go to the Templates tab to install one.</div>"#);
    } else {
        for s in &statuses {
            html.push_str(&super::services_row::render_service_row(s, &config, &host_ips));
        }
    }

    html.push_str("</div>");
    html
}
