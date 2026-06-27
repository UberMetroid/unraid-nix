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

    let mut html = r#"<div style="overflow-x: auto; width: 100%;">
        <table class="nix-services-table">
            <thead>
                <tr>
                    <th>Application</th>
                    <th>IP:Port</th>
                    <th>Volume Mappings</th>
                    <th>Resources</th>
                    <th>Actions</th>
                    <th>Autostart</th>
                </tr>
            </thead>
            <tbody>"#.to_string();

    if statuses.is_empty() {
        html.push_str(r#"<tr><td colspan="6" class="text-center">No Nix Flake services configured. Go to the Flakes tab to install one.</td></tr>"#);
    } else {
        for s in &statuses {
            html.push_str(&super::services_row::render_service_row(s, &config, &host_ips));
        }
    }

    html.push_str("</tbody></table></div>");
    html
}
