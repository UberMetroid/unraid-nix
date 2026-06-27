use crate::process::{get_services_status, is_supervisor_running};

fn get_system_total_memory_kb() -> u64 {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb;
                    }
                }
            }
        }
    }
    32 * 1024 * 1024 // Fallback if missing
}

fn get_service_gpus(name: &str) -> String {
    let metadata_file = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(gpus) = meta.get("gpus").and_then(|v| v.as_str()) {
                if !gpus.trim().is_empty() {
                    return gpus.to_string();
                }
            }
            if let Some(gpu) = meta.get("gpu").and_then(|v| v.as_str()) {
                if gpu == "1" || gpu == "true" {
                    return "All GPUs".to_string();
                }
            }
        }
    }
    "-".to_string()
}

/// Renders a dashboard widget summary showing all active Nix services.
pub fn render_dashboard_widget(api_port: u16) -> String {
    if !is_supervisor_running() {
        return r#"<div style="padding: 15px; text-align: center; color: #a0a0a5; font-size: 12px;">
            <i class="fa fa-exclamation-triangle" style="margin-right: 5px; color: #f39c12;"></i> Nix supervisor not running.
        </div>"#.to_string();
    }

    let statuses = match get_services_status(api_port) {
        Ok(mut s) => {
            s.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            s
        }
        Err(_) => return r#"<div style="padding: 15px; text-align: center; color: #e74c3c; font-size: 12px;">
            <i class="fa fa-times" style="margin-right: 5px;"></i> Error reading service statuses.
        </div>"#.to_string(),
    };

    if statuses.is_empty() {
        return r#"<div style="padding: 15px; text-align: center; color: #a0a0a5; font-size: 12px;">
            No Nix Flake services configured.
        </div>"#.to_string();
    }

    let mut running_count = 0;
    let mut total_cpu = 0.0;
    let mut total_mem_bytes = 0;

    for s in &statuses {
        if s.status.to_lowercase() == "running" {
            running_count += 1;
            total_cpu += s.cpu.unwrap_or(0.0);
            total_mem_bytes += s.memory.unwrap_or(0);
        }
    }

    let total_count = statuses.len();
    let total_mem_mb = total_mem_bytes as f64 / 1_048_576.0;

    let sys_mem_kb = get_system_total_memory_kb();
    let sys_mem_bytes = sys_mem_kb * 1024;
    let mem_percentage = if sys_mem_bytes > 0 {
        (total_mem_bytes as f64 / sys_mem_bytes as f64) * 100.0
    } else {
        0.0
    };

    let cpu_bar_width = total_cpu.min(100.0);
    let mem_bar_width = mem_percentage.min(100.0);

    let mut html = format!(
        r#"<style>
            .nix-dash-widget {{
                font-family: "Clear Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
                color: #eee;
            }}
            .nix-dash-summary {{
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 15px;
                background: rgba(255, 255, 255, 0.02);
                border: 1px solid rgba(255, 255, 255, 0.05);
                border-radius: 4px;
                padding: 10px;
                margin-bottom: 12px;
            }}
            .nix-dash-metric {{
                display: flex;
                flex-direction: column;
                gap: 4px;
            }}
            .nix-dash-metric-header {{
                display: flex;
                justify-content: space-between;
                font-size: 11px;
                color: #a0a0a5;
                font-weight: 500;
            }}
            .nix-dash-progress-bg {{
                height: 6px;
                background: rgba(255, 255, 255, 0.08);
                border-radius: 3px;
                overflow: hidden;
            }}
            .nix-dash-progress-bar {{
                height: 100%;
                background: #00a1ff;
                border-radius: 3px;
                transition: width 0.4s ease;
            }}
            .nix-dash-table {{
                width: 100%;
                border-collapse: collapse;
                font-size: 11px;
            }}
            .nix-dash-table th {{
                text-align: left;
                color: #888;
                font-weight: 600;
                padding: 6px 8px;
                border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            }}
            .nix-dash-table td {{
                padding: 8px;
                border-bottom: 1px solid rgba(255, 255, 255, 0.03);
                vertical-align: middle;
            }}
            .nix-dash-status-dot {{
                display: inline-block;
                width: 7px;
                height: 7px;
                border-radius: 50%;
                margin-right: 6px;
            }}
            .nix-dash-status-dot.running {{
                background: #2ecc71;
                box-shadow: 0 0 6px #2ecc71;
            }}
            .nix-dash-status-dot.stopped {{
                background: #e74c3c;
            }}
            .nix-dash-gpu-badge {{
                background: rgba(0, 161, 255, 0.08);
                border: 1px solid rgba(0, 161, 255, 0.25);
                border-radius: 3px;
                padding: 1px 4px;
                font-size: 9px;
                color: #00a1ff;
                font-family: monospace;
                display: inline-block;
                margin-bottom: 2px;
            }}
        </style>
        <div class="nix-dash-widget">
            <div style="font-size: 12px; font-weight: 600; color: #eee; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                <span>Nix Services Overview</span>
                <span style="font-weight: 500; font-size: 11px; color: #a0a0a5;">{} / {} Running</span>
            </div>
            
            <div class="nix-dash-summary">
                <div class="nix-dash-metric">
                    <div class="nix-dash-metric-header">
                        <span>TOTAL CPU</span>
                        <span style="font-family: monospace; color: #eee;">{:.1}%</span>
                    </div>
                    <div class="nix-dash-progress-bg">
                        <div class="nix-dash-progress-bar" style="width: {:.1}%;"></div>
                    </div>
                </div>
                <div class="nix-dash-metric">
                    <div class="nix-dash-metric-header">
                        <span>TOTAL MEMORY</span>
                        <span style="font-family: monospace; color: #eee;">{:.1} MB</span>
                    </div>
                    <div class="nix-dash-progress-bg">
                        <div class="nix-dash-progress-bar" style="width: {:.1}%;"></div>
                    </div>
                </div>
            </div>

            <table class="nix-dash-table">
                <thead>
                    <tr>
                        <th>Service</th>
                        <th>GPU</th>
                        <th>CPU</th>
                        <th>Memory</th>
                        <th>Uptime</th>
                    </tr>
                </thead>
                <tbody>"#,
        running_count, total_count, total_cpu, cpu_bar_width, total_mem_mb, mem_bar_width
    );

    for s in statuses {
        let is_running = s.status.to_lowercase() == "running";
        let status_class = if is_running { "running" } else { "stopped" };
        
        let cpu_str = if is_running {
            s.cpu.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "0.0%".to_string())
        } else {
            "-".to_string()
        };

        let mem_str = if is_running {
            s.memory.map(|m| format!("{:.1} MB", m as f64 / 1_048_576.0)).unwrap_or_else(|| "0.0 MB".to_string())
        } else {
            "-".to_string()
        };

        let uptime_str = if is_running {
            s.uptime()
        } else {
            "-".to_string()
        };

        let gpus = get_service_gpus(&s.name);
        let gpus_html = if gpus == "-" || gpus.is_empty() {
            r#"<span style="color: #666;">-</span>"#.to_string()
        } else {
            let mut badges = Vec::new();
            for part in gpus.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    let display_part = if p.starts_with("nvidia-") {
                        p.replace("nvidia-", "GPU-")
                    } else {
                        p.to_string()
                    };
                    badges.push(format!(
                        r#"<div style="margin-bottom: 2px;"><span class="nix-dash-gpu-badge">{}</span></div>"#,
                        display_part
                    ));
                }
            }
            if badges.is_empty() {
                r#"<span style="color: #666;">-</span>"#.to_string()
            } else {
                badges.join("")
            }
        };

        html.push_str(&format!(
            r#"<tr>
                <td style="font-weight: 500;"><span class="nix-dash-status-dot {}"></span>{}</td>
                <td>{}</td>
                <td style="font-family: monospace; color: #eee;">{}</td>
                <td style="font-family: monospace; color: #eee;">{}</td>
                <td style="color: #a0a0a5;">{}</td>
            </tr>"#,
            status_class, s.name, gpus_html, cpu_str, mem_str, uptime_str
        ));
    }

    html.push_str("</tbody></table></div>");
    
    html.push_str(
        r#"<script>
        if (typeof window.nixDashTimer === 'undefined') {
            window.nixDashTimer = setInterval(function() {
                var container = document.getElementById('nix-dashboard-container');
                if (!container) {
                    clearInterval(window.nixDashTimer);
                    delete window.nixDashTimer;
                    return;
                }
                fetch('/plugins/nix/api.php?action=get_dashboard')
                    .then(function(resp) { return resp.text(); })
                    .then(function(html) {
                        if (html.trim() !== '') {
                            container.innerHTML = html;
                        }
                    })
                    .catch(function(err) { console.error('Error refreshing dashboard:', err); });
            }, 3000);
        }
        </script>"#
    );

    html
}
