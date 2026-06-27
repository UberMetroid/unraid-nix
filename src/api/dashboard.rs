use crate::process::{get_services_status, is_supervisor_running};

fn get_sorted_statuses(api_port: u16) -> Result<Vec<crate::process::ServiceStatus>, String> {
    let mut statuses = get_services_status(api_port)?;
    statuses.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(statuses)
}

/// Renders ONLY the service rows as HTML (used for initial widget render).
pub fn render_dashboard_rows(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<tr><td><span class="left" style="color: #999;">Supervisor not running</span></td></tr>"#.to_string();
    }

    let statuses = match get_sorted_statuses(api_port) {
        Ok(s) => s,
        Err(_) => return r#"<tr><td><span class="left" style="color: #e74c3c;">Error reading statuses</span></td></tr>"#.to_string(),
    };

    if statuses.is_empty() {
        return r#"<tr><td><span class="left" style="color: #999;">No configured services</span></td></tr>"#.to_string();
    }

    let mut html = String::new();
    for s in statuses {
        let is_running = s.status.to_lowercase() == "running";
        let status_text = if is_running { "Running" } else { "Stopped" };
        let status_color = if is_running { "#2ecc71" } else { "#e74c3c" };
        let shadow = if is_running { "0 0 5px #2ecc71" } else { "none" };

        html.push_str(&format!(
            r#"<tr data-service="{}">
                <td>
                    <span class="left">{}</span>
                    <span class="right">
                        <span class="status-dot" style="background: {}; display: inline-block; width: 8px; height: 8px; border-radius: 50%; margin-right: 5px; box-shadow: {};"></span>
                        <span class="status-text">{}</span>
                    </span>
                </td>
            </tr>"#,
            s.name, s.name, status_color, shadow, status_text
        ));
    }
    html
}

/// Returns the service statuses directly as a JSON string (used for dynamic updates).
pub fn render_dashboard_json(api_port: u16) -> String {
    if let Ok(statuses) = get_sorted_statuses(api_port) {
        serde_json::to_string(&statuses).unwrap_or_else(|_| "[]".to_string())
    } else {
        "[]".to_string()
    }
}

/// Renders the complete native Unraid Dashboard widget tile structure (<tbody>).
pub fn render_dashboard_widget(api_port: u16) -> String {
    let rows_html = render_dashboard_rows(api_port);
    let mut html = String::new();
    
    html.push_str(r#"<tbody title="Nix Services">
        <tr>
            <td>
                <div class="tile-header">
                    <div class="tile-header-left">
                        <i class="fa fa-snowflake-o" style="margin-right: 8px; font-size: 18px; color: #00a1ff; vertical-align: middle;"></i>
                        <span class="section" style="vertical-align: middle;">Nix Services</span>
                    </div>
                    <div class="tile-header-right">
                        <a href="/Nix/nix_services" title="Settings"><i class="fa fa-fw fa-cog control"></i></a>
                    </div>
                </div>
            </td>
        </tr>"#);
        
    html.push_str(&rows_html);
    
    html.push_str(r#"</tbody>
    <script>
    if (typeof window.nixDashTimer === 'undefined') {
        window.nixDashTimer = setInterval(function() {
            var tbody = document.querySelector('tbody[title="Nix Services"]');
            if (!tbody) {
                clearInterval(window.nixDashTimer);
                delete window.nixDashTimer;
                return;
            }
            fetch('/plugins/nix/api.php?action=get_dashboard_json')
                .then(function(resp) { return resp.json(); })
                .then(function(services) {
                    if (Array.isArray(services)) {
                        services.forEach(function(s) {
                            var row = tbody.querySelector('tr[data-service="' + s.name + '"]');
                            if (row) {
                                var isRunning = s.status.toLowerCase() === 'running';
                                var dot = row.querySelector('.status-dot');
                                var txt = row.querySelector('.status-text');
                                if (dot) {
                                    dot.style.background = isRunning ? '#2ecc71' : '#e74c3c';
                                    dot.style.boxShadow = isRunning ? '0 0 5px #2ecc71' : 'none';
                                }
                                if (txt) {
                                    txt.textContent = isRunning ? 'Running' : 'Stopped';
                                }
                            }
                        });
                    }
                })
                .catch(function(err) { console.error('Error refreshing dashboard json:', err); });
        }, 3000);
    }
    </script>"#);

    html
}
