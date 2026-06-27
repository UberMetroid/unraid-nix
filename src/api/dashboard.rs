use crate::process::{get_services_status, is_supervisor_running};

fn get_sorted_statuses(api_port: u16) -> Result<Vec<crate::process::ServiceStatus>, String> {
    let mut statuses = get_services_status(api_port)?;
    statuses.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(statuses)
}

/// Renders ONLY the service rows for the dashboard widget (used for AJAX live auto-refresh).
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
            r#"<tr>
                <td>
                    <span class="left">{}</span>
                    <span class="right">
                        <span class="status-dot" style="background: {}; display: inline-block; width: 8px; height: 8px; border-radius: 50%; margin-right: 5px; box-shadow: {};"></span>
                        {}
                    </span>
                </td>
            </tr>"#,
            s.name, status_color, shadow, status_text
        ));
    }
    html
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
            fetch('/plugins/nix/api.php?action=get_dashboard_rows')
                .then(function(resp) { return resp.text(); })
                .then(function(html) {
                    if (html.trim() !== '') {
                        // Clear all rows except the first header row
                        while (tbody.rows.length > 1) {
                            tbody.deleteRow(1);
                        }
                        // Parse rows within a table context to ensure correct browser DOM injection
                        var temp = document.createElement('div');
                        temp.innerHTML = '<table><tbody>' + html + '</tbody></table>';
                        var rows = temp.querySelector('tbody').rows;
                        while (rows.length > 0) {
                            tbody.appendChild(rows[0]);
                        }
                    }
                })
                .catch(function(err) { console.error('Error refreshing dashboard rows:', err); });
        }, 3000);
    }
    </script>"#);

    html
}
