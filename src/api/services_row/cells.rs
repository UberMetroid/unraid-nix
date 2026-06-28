use crate::api::utils::HostAddr;
use crate::api::utils::{html_escape, js_escape};
use crate::process::GpuStat;

pub fn render_lan_ip_port_cell(
    port_num: Option<u16>,
    is_running: bool,
    host_ips: &[HostAddr],
    bind_address_override: &Option<String>,
) -> String {
    if let Some(port) = port_num {
        let mut ip_links = Vec::new();
        let has_specific_bind = if let Some(ref addr) = bind_address_override {
            let a = addr.trim();
            !a.is_empty() && a != "0.0.0.0" && a != "*"
        } else {
            false
        };

        for addr in host_ips {
            if has_specific_bind {
                if let Some(ref target) = bind_address_override {
                    if addr.ip != target.trim() {
                        continue;
                    }
                }
            }

            let link = if is_running {
                format!(
                    r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: var(--nix-accent); text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a></div>"##,
                    html_escape(&js_escape(&addr.ip)),
                    port,
                    html_escape(&addr.ip),
                    port
                )
            } else {
                format!(
                    r##"<div style="margin-bottom: 4px;"><span style="color: var(--nix-text-secondary);">{}:{}</span></div>"##,
                    html_escape(&addr.ip),
                    port
                )
            };
            ip_links.push(link);
        }

        if ip_links.is_empty() && has_specific_bind {
            if let Some(ref target) = bind_address_override {
                let link = if is_running {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: var(--nix-accent); text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a></div>"##,
                        html_escape(&js_escape(target)),
                        port,
                        html_escape(target),
                        port
                    )
                } else {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><span style="color: var(--nix-text-secondary);">{}:{}</span></div>"##,
                        html_escape(target),
                        port
                    )
                };
                ip_links.push(link);
            }
        }

        if ip_links.is_empty() {
            "-".to_string()
        } else {
            ip_links.join("")
        }
    } else {
        "-".to_string()
    }
}

pub fn render_resources_cell(
    name: &str,
    is_running: bool,
    cpu: Option<f32>,
    memory: Option<u64>,
    gpus_override: &Option<String>,
    legacy_gpu: &Option<String>,
    _gpu_stats: &Option<std::collections::HashMap<i32, GpuStat>>,
) -> String {
    let mut res = String::new();
    if is_running {
        let cpu_val = cpu.unwrap_or(0.0);
        let mem_val = memory.unwrap_or(0);
        let mb = mem_val as f64 / 1_048_576.0;

        let cpu_str = format!("{:.1}%", cpu_val);
        let mem_str = format!("{:.1} MB", mb);

        let has_gpu = gpus_override
            .as_ref()
            .map(|g| !g.trim().is_empty())
            .unwrap_or(false)
            || legacy_gpu
                .as_ref()
                .map(|l| l == "1" || l == "true")
                .unwrap_or(false);

        let gpu_html = if has_gpu {
            let mut gpu_sm = 0;
            let mut gpu_mem = 0;
            if let Some(ref stats) = _gpu_stats {
                if let Some(stat) = stats.values().next() {
                    gpu_sm = stat.sm;
                    gpu_mem = stat.mem;
                }
            }
            format!(
                r#"<div class="nix-stat-row" data-service="{}" data-type="gpu" style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="color: var(--nix-text-secondary); font-size: 10px;">GPU</span>
                    <span class="nix-stat-val" style="color: #10b981; font-family: monospace; font-weight: 500;">{}%</span>
                </div>
                <div class="nix-stat-row" data-service="{}" data-type="gpu-mem" style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="color: var(--nix-text-secondary); font-size: 10px;">VRAM</span>
                    <span class="nix-stat-val" style="color: #a855f7; font-family: monospace; font-weight: 500;">{}%</span>
                </div>"#,
                html_escape(name),
                gpu_sm,
                html_escape(name),
                gpu_mem
            )
        } else {
            "".to_string()
        };

        res.push_str(&format!(
            r#"<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 4px 16px; font-size: 11px; width: 100%; box-sizing: border-box;">
                <!-- Row 1 -->
                <div class="nix-stat-row" data-service="{}" data-type="cpu" style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="color: var(--nix-text-secondary); font-size: 10px;">CPU</span>
                    <span class="nix-stat-val" style="color: #00d5ff; font-family: monospace; font-weight: 500;">{}</span>
                </div>
                <div class="nix-stat-row" data-service="{}" data-type="io-in" style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="color: var(--nix-text-secondary); font-size: 10px; display: inline-flex; align-items: center; gap: 2px;"><i class="fa fa-arrow-down" style="color: #2ecc71; font-size: 8px;"></i> IN</span>
                    <span class="nix-stat-val" style="color: #2ecc71; font-family: monospace; font-weight: 500;">0.0 B/s</span>
                </div>
                <!-- Row 2 -->
                <div class="nix-stat-row" data-service="{}" data-type="ram" style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="color: var(--nix-text-secondary); font-size: 10px;">RAM</span>
                    <span class="nix-stat-val" style="color: #d946ef; font-family: monospace; font-weight: 500;">{}</span>
                </div>
                <div class="nix-stat-row" data-service="{}" data-type="io-out" style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="color: var(--nix-text-secondary); font-size: 10px; display: inline-flex; align-items: center; gap: 2px;"><i class="fa fa-arrow-up" style="color: #e67e22; font-size: 8px;"></i> OUT</span>
                    <span class="nix-stat-val" style="color: #e67e22; font-family: monospace; font-weight: 500;">0.0 B/s</span>
                </div>
                <!-- Row 3 -->
                {}
            </div>"#,
            html_escape(name), cpu_str, html_escape(name), html_escape(name), mem_str, html_escape(name), gpu_html
        ));
    } else {
        res.push_str(r#"<span style="color: var(--nix-text-muted);">-</span>"#);
    }
    res
}

pub fn render_autostart_cell(name: &str, autostart_enabled: bool) -> String {
    if autostart_enabled {
        format!(
            r#"<button type="button" class="nix-btn nix-btn-sm" onclick="toggleAutostart('{}', false)" title="Autostart: Enabled"><i class="fa fa-toggle-on" style="color: #2ecc71;"></i></button>"#,
            html_escape(&js_escape(name))
        )
    } else {
        format!(
            r#"<button type="button" class="nix-btn nix-btn-sm" onclick="toggleAutostart('{}', true)" title="Autostart: Disabled"><i class="fa fa-toggle-off" style="color: var(--nix-text-muted);"></i></button>"#,
            html_escape(&js_escape(name))
        )
    }
}
