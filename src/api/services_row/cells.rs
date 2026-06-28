use crate::api::utils::HostAddr;
use crate::process::GpuStat;
use super::static_config::get_service_fa_config;

#[allow(dead_code)]
pub fn render_app_cell(name: &str, version_badge: &str, status_subtext: &str) -> String {
    let cfg = get_service_fa_config(name);
    format!(
        r#"<div style="display: flex; align-items: center; gap: 10px;">
            <div style="width: 28px; height: 28px; border-radius: 4px; background: {}; border: 1px solid {}; display: inline-flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0;">
                <i class="fa {}" style="font-size: 14px;"></i>
            </div>
            <div style="display: flex; flex-direction: column; gap: 2px;">
                <strong style="font-size: 13px;">{}</strong>
                {}
                <div style="font-size: 11px; color: var(--nix-text-secondary);">{}</div>
            </div>
        </div>"#,
        cfg.bg, cfg.border, cfg.color, cfg.icon, name, version_badge, status_subtext
    )
}

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
                    addr.ip, port, addr.ip, port
                )
            } else {
                format!(
                    r##"<div style="margin-bottom: 4px;"><span style="color: var(--nix-text-secondary);">{}:{}</span></div>"##,
                    addr.ip, port
                )
            };
            ip_links.push(link);
        }

        if ip_links.is_empty() && has_specific_bind {
            if let Some(ref target) = bind_address_override {
                let link = if is_running {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: var(--nix-accent); text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a></div>"##,
                        target, port, target, port
                    )
                } else {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><span style="color: var(--nix-text-secondary);">{}:{}</span></div>"##,
                        target, port
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
    _gpus_override: &Option<String>,
    _legacy_gpu: &Option<String>,
    _gpu_stats: &Option<std::collections::HashMap<i32, GpuStat>>,
) -> String {
    let mut res = String::new();
    if is_running {
        let cpu_val = cpu.unwrap_or(0.0);
        let mem_val = memory.unwrap_or(0);
        let mb = mem_val as f64 / 1_048_576.0;
        
        let cpu_str = format!("{:.1}%", cpu_val);
        let mem_str = format!("{:.1} MB", mb);
        
        res.push_str(&format!(
            r#"<div style="display: flex; flex-direction: column; align-items: flex-end; gap: 4px;">
                <div>
                    <div class="nix-stat-row" data-service="{}" data-type="cpu" style="display: inline-block; margin-right: 6px;">
                        <span class="nix-stat-val" style="font-size: 11px; color: #00d5ff; font-family: monospace; font-weight: 500;">{}</span> <span style="font-size: 10px; color: var(--nix-text-muted);">CPU</span>
                    </div>
                    <div class="nix-stat-row" data-service="{}" data-type="ram" style="display: inline-block;">
                        <span class="nix-stat-val" style="font-size: 11px; color: #d946ef; font-family: monospace; font-weight: 500;">{}</span> <span style="font-size: 10px; color: var(--nix-text-muted);">RAM</span>
                    </div>
                </div>
                <div>
                    <div class="nix-stat-row" data-service="{}" data-type="io-in" style="display: inline-block; margin-right: 6px;">
                        <i class="fa fa-arrow-down" style="color: #2ecc71; font-size: 9px; margin-right: 1px;"></i><span class="nix-stat-val" style="font-size: 11px; color: #2ecc71; font-family: monospace; font-weight: 500;">0.0 B/s</span>
                    </div>
                    <div class="nix-stat-row" data-service="{}" data-type="io-out" style="display: inline-block;">
                        <i class="fa fa-arrow-up" style="color: #e67e22; font-size: 9px; margin-right: 1px;"></i><span class="nix-stat-val" style="font-size: 11px; color: #e67e22; font-family: monospace; font-weight: 500;">0.0 B/s</span>
                    </div>
                </div>
               </div>"#,
            name, cpu_str, name, mem_str, name, name
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
            name
        )
    } else {
        format!(
            r#"<button type="button" class="nix-btn nix-btn-sm" onclick="toggleAutostart('{}', true)" title="Autostart: Disabled"><i class="fa fa-toggle-off" style="color: var(--nix-text-muted);"></i></button>"#,
            name
        )
    }
}
