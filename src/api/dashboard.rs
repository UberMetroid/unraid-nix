use crate::process::{get_services_status, is_supervisor_running};
use crate::api::utils::{get_service_web_port, get_host_ips};
use crate::api::utils::{html_escape, js_escape};

fn get_sorted_statuses(api_port: u16) -> Result<Vec<crate::process::ServiceStatus>, String> {
    let mut statuses = get_services_status(api_port)?;
    statuses.sort_by_key(|a| a.name.to_lowercase());
    Ok(statuses)
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

    let host_ips = get_host_ips();
    let ip_str = host_ips.first().map(|h| h.ip.as_str()).unwrap_or("127.0.0.1");

    let mut html = String::new();
    for s in statuses {
        let is_running = s.status.to_lowercase() == "running";
        let status_text = if is_running { "Running" } else { "Stopped" };
        let status_color = if is_running { "#2ecc71" } else { "#e74c3c" };
        let shadow = if is_running { "0 0 5px #2ecc71" } else { "none" };

        let port = get_service_web_port(&s.name);
        let name_link_html = if let Some(p) = port {
            format!(
                r#"<a href="http://{}:{p}/" target="_blank" title="Open Web UI" style="text-decoration: none; color: inherit; display: inline-flex; align-items: center; gap: 6px;">
                    <img src="/plugins/nix/api.php?action=get-icon&service={}" style="width: 16px; height: 16px; border-radius: 2px; vertical-align: middle;" />
                    <span style="vertical-align: middle; font-weight: 500;">{}</span>
                    <i class="fa fa-external-link" style="font-size: 8px; color: #00a1ff; opacity: 0.7; vertical-align: middle;"></i>
                </a>"#,
                html_escape(ip_str), html_escape(&s.name), html_escape(&s.name)
            )
        } else {
            format!(
                r#"<span style="display: inline-flex; align-items: center; gap: 6px;">
                    <img src="/plugins/nix/api.php?action=get-icon&service={}" style="width: 16px; height: 16px; border-radius: 2px; vertical-align: middle;" />
                    <span style="vertical-align: middle; font-weight: 500;">{}</span>
                </span>"#,
                html_escape(&s.name), html_escape(&s.name)
            )
        };

        let btn_icon = if is_running { "fa-stop" } else { "fa-play" };
        let btn_action = if is_running { "stop" } else { "start" };
        let btn_title = if is_running { "Stop Service" } else { "Start Service" };

        let gpu_indicator = if s.gpu_active.unwrap_or(false) {
            r#"<i class="fa fa-microchip nix-dash-gpu-active" style="font-size: 11px; color: #00a1ff; vertical-align: middle;" title="GPU Active"></i>"#
        } else {
            r#"<span style="color: #666;">-</span>"#
        };

        html.push_str(&format!(
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
            html_escape(&s.name), name_link_html, status_color, shadow, status_text, gpu_indicator, html_escape(&js_escape(&s.name)), btn_action, btn_title, btn_icon
        ));
    }
    html
}

/// Returns the service statuses directly as a JSON string (used for dynamic updates).
pub fn render_dashboard_json(api_port: u16) -> String {
    get_sorted_statuses(api_port)
        .map(|statuses| serde_json::to_string(&statuses).unwrap_or_else(|_| "[]".to_string()))
        .unwrap_or_else(|_| "[]".to_string())
}

/// Renders the complete native Unraid Dashboard widget tile structure (<table>).
pub fn render_dashboard_widget(api_port: u16) -> String {
    let rows_html = render_dashboard_rows(api_port);
    let mut html = String::new();
    
    html.push_str(r#"<table class="nix-dash-table" style="width: 100%; border-collapse: collapse; margin-top: 5px;">
        <thead>
            <tr style="border-bottom: 1px solid rgba(255, 255, 255, 0.08); text-align: left; font-size: 10px; color: #a0a0a5; text-transform: uppercase; letter-spacing: 0.5px;">
                <th style="padding: 6px 8px;">Service</th>
                <th style="padding: 6px 8px;">Status</th>
                <th style="padding: 6px 8px; text-align: center;">GPU</th>
                <th style="padding: 6px 8px; text-align: right;">Action</th>
            </tr>
        </thead>
        <tbody class="nix-dash-rows">"#);
        
    html.push_str(&rows_html);
    
    html.push_str(r#"</tbody>
    </table>
    <style>
    .nix-dash-table tr { background: transparent; transition: background 0.15s ease; }
    .nix-dash-table tr:hover { background: rgba(255, 255, 255, 0.015); }
    .nix-dash-toggle-btn:hover i.fa-play { color: #2ecc71 !important; text-shadow: 0 0 4px #2ecc71; }
    .nix-dash-toggle-btn:hover i.fa-stop { color: #e74c3c !important; text-shadow: 0 0 4px #e74c3c; }
    .nix-dash-gpu-active {
        animation: nix-gpu-pulse 1.6s infinite ease-in-out;
        color: #00a1ff !important;
        text-shadow: 0 0 6px #00a1ff, 0 0 12px rgba(0, 161, 255, 0.4);
        display: inline-block;
    }
    @keyframes nix-gpu-pulse {
        0% { transform: scale(1); opacity: 1; }
        50% { transform: scale(1.25); opacity: 0.65; color: #00e5ff !important; }
        100% { transform: scale(1); opacity: 1; }
    }
    </style>
    <script>
    if (typeof window.toggleDashService === 'undefined') {
        window.toggleDashService = function(name, action) {
            var btn = event.currentTarget;
            var icon = btn.querySelector('i');
            if (icon.classList.contains('fa-spinner')) return;
            
            icon.className = 'fa fa-spinner fa-spin';
            btn.disabled = true;
            
            var params = new URLSearchParams();
            params.append('service', name);
            
            fetch('/plugins/nix/api.php?action=' + action, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                body: params.toString()
            })
            .then(function(resp) { return resp.json(); })
            .then(function(data) {
                if (typeof window.refreshNixDash === 'function') {
                    window.refreshNixDash();
                }
            })
            .catch(function(err) {
                console.error('Error toggling service:', err);
                btn.disabled = false;
                icon.className = 'fa ' + (action === 'start' ? 'fa-play' : 'fa-stop');
            });
        };
    }

    if (typeof window.refreshNixDash === 'undefined') {
        window.refreshNixDash = function() {
            var tbody = document.querySelector('tbody.nix-dash-rows');
            if (!tbody) {
                clearInterval(window.nixDashTimer);
                delete window.nixDashTimer;
                delete window.refreshNixDash;
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
                                var gpuWrap = row.querySelector('.gpu-wrapper');
                                var btn = row.querySelector('.nix-dash-toggle-btn');
                                var btnIcon = btn ? btn.querySelector('i') : null;
                                
                                if (dot) {
                                    dot.style.background = isRunning ? '#2ecc71' : '#e74c3c';
                                    dot.style.boxShadow = isRunning ? '0 0 5px #2ecc71' : 'none';
                                }
                                if (txt) {
                                    txt.textContent = isRunning ? 'Running' : 'Stopped';
                                }
                                if (gpuWrap) {
                                    if (s.gpu_active) {
                                        if (!gpuWrap.querySelector('.nix-dash-gpu-active')) {
                                            gpuWrap.innerHTML = '<i class="fa fa-microchip nix-dash-gpu-active" style="font-size: 11px; color: #00a1ff; vertical-align: middle;" title="GPU Active"></i>';
                                        }
                                    } else {
                                        gpuWrap.innerHTML = '<span style="color: #666;">-</span>';
                                    }
                                }
                                if (btn && btnIcon && !btnIcon.classList.contains('fa-spinner')) {
                                    btn.disabled = false;
                                    btnIcon.className = 'fa ' + (isRunning ? 'fa-stop' : 'fa-play');
                                    btn.title = isRunning ? 'Stop Service' : 'Start Service';
                                    btn.setAttribute('onclick', "toggleDashService('" + s.name + "', '" + (isRunning ? 'stop' : 'start') + "')");
                                }
                            }
                        });
                    }
                })
                .catch(function(err) { console.error('Error refreshing dashboard json:', err); });
        };
        
        window.nixDashTimer = setInterval(window.refreshNixDash, 3000);
    }
    </script>"#);

    html
}
