use crate::api::utils::HostAddr;
use super::static_config::get_service_fa_config;

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
                <div style="font-size: 11px; color: #a0a0a5;">{}</div>
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

            let label = match addr.interface.to_lowercase().as_str() {
                "tailscale0" | "tailscale" => "tailscale".to_string(),
                other => other.to_string(),
            };
            
            let link = if is_running {
                format!(
                    r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a> <span style="font-size: 10px; color: #777; font-family: monospace;">({})</span></div>"##,
                    addr.ip, port, addr.ip, port, label
                )
            } else {
                format!(
                    r##"<div style="margin-bottom: 4px;"><span style="color: #888;">{}:{}</span> <span style="font-size: 10px; color: #555; font-family: monospace;">({})</span></div>"##,
                    addr.ip, port, label
                )
            };
            ip_links.push(link);
        }

        if ip_links.is_empty() && has_specific_bind {
            if let Some(ref target) = bind_address_override {
                let link = if is_running {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a> <span style="font-size: 10px; color: #777; font-family: monospace;">(override)</span></div>"##,
                        target, port, target, port
                    )
                } else {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><span style="color: #888;">{}:{}</span> <span style="font-size: 10px; color: #555; font-family: monospace;">(override)</span></div>"##,
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

pub fn render_volume_mappings_cell(home_path: &str, extra_binds_vec: &[(String, String)]) -> String {
    let mut volume_mappings = Vec::new();
    if home_path != "-" && !home_path.is_empty() {
        volume_mappings.push(format!(
            r#"<div style="margin-bottom: 4px;"><span style="color: #a0a0a5; font-family: monospace;">/config</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code></div>"#,
            home_path
        ));
    }
    for (host, sandbox) in extra_binds_vec {
        if !host.is_empty() && !sandbox.is_empty() {
            volume_mappings.push(format!(
                r#"<div style="margin-bottom: 4px;"><span style="color: #a0a0a5; font-family: monospace;">{}</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code></div>"#,
                sandbox, host
            ));
        }
    }

    if volume_mappings.is_empty() {
        "-".to_string()
    } else {
        volume_mappings.join("")
    }
}

pub fn render_resources_cell(
    name: &str,
    is_running: bool,
    cpu: Option<f32>,
    memory: Option<u64>,
    gpus_override: &Option<String>,
    legacy_gpu: &Option<String>,
) -> String {
    let gpus_display = match gpus_override {
        Some(ref g) if !g.trim().is_empty() => {
            let mut badges = Vec::new();
            for part in g.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    let display_part = if p.starts_with("nvidia-") {
                        p.replace("nvidia-", "GPU-")
                    } else {
                        p.to_string()
                    };
                    badges.push(format!(
                        r#"<span style="background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.25); border-radius: 3px; padding: 2px 6px; font-size: 10px; color: #00a1ff; font-family: monospace; display: inline-block;">{}</span>"#,
                        display_part
                    ));
                }
            }
            if badges.is_empty() {
                r#"<span style="color: #777;">-</span>"#.to_string()
            } else {
                format!(
                    r#"<div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px;">{}</div>"#,
                    badges.join("")
                )
            }
        }
        _ => {
            if let Some(ref lg) = legacy_gpu {
                if lg == "1" || lg == "true" {
                    r#"<div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px;"><span style="background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.25); border-radius: 3px; padding: 2px 6px; font-size: 10px; color: #00a1ff; font-family: monospace; display: inline-block;">All GPUs</span></div>"#.to_string()
                } else {
                    r#"<span style="color: #777;">-</span>"#.to_string()
                }
            } else {
                r#"<span style="color: #777;">-</span>"#.to_string()
            }
        }
    };

    let mut res = String::new();
    if is_running {
        let cpu_val = cpu.unwrap_or(0.0);
        let mem_val = memory.unwrap_or(0);
        let mb = mem_val as f64 / 1_048_576.0;
        
        let cpu_str = format!("{:.1}%", cpu_val);
        let mem_str = format!("{:.1} MB", mb);
        
        res.push_str(&format!(
            r#"<div class="nix-stat-row" data-service="{}" data-type="cpu" style="display: flex; align-items: center; gap: 8px; margin-bottom: 2px;">
                <svg class="nix-sparkline" style="width: 60px; height: 12px; overflow: visible; display: inline-block; vertical-align: middle;"></svg>
                <span class="nix-stat-val" style="font-size: 11px; color: #00d5ff; font-family: monospace; font-weight: 500; min-width: 45px; text-align: right; display: inline-block;">{}</span>
                <span style="font-size: 10px; color: #666; font-family: monospace;">CPU</span>
               </div>
               <div class="nix-stat-row" data-service="{}" data-type="ram" style="display: flex; align-items: center; gap: 8px; margin-bottom: 2px;">
                <svg class="nix-sparkline" style="width: 60px; height: 12px; overflow: visible; display: inline-block; vertical-align: middle;"></svg>
                <span class="nix-stat-val" style="font-size: 11px; color: #d946ef; font-family: monospace; font-weight: 500; min-width: 45px; text-align: right; display: inline-block;">{}</span>
                <span style="font-size: 10px; color: #666; font-family: monospace;">RAM</span>
               </div>
               <div class="nix-stat-row" data-service="{}" data-type="io-in" style="display: flex; align-items: center; gap: 8px; margin-bottom: 2px;">
                <svg class="nix-sparkline" style="width: 60px; height: 12px; overflow: visible; display: inline-block; vertical-align: middle;"></svg>
                <span class="nix-stat-val" style="font-size: 11px; color: #2ecc71; font-family: monospace; font-weight: 500; min-width: 45px; text-align: right; display: inline-block;">0.0 B/s</span>
                <span style="font-size: 10px; color: #666; font-family: monospace;">I/O In</span>
               </div>
               <div class="nix-stat-row" data-service="{}" data-type="io-out" style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                <svg class="nix-sparkline" style="width: 60px; height: 12px; overflow: visible; display: inline-block; vertical-align: middle;"></svg>
                <span class="nix-stat-val" style="font-size: 11px; color: #e67e22; font-family: monospace; font-weight: 500; min-width: 45px; text-align: right; display: inline-block;">0.0 B/s</span>
                <span style="font-size: 10px; color: #666; font-family: monospace;">I/O Out</span>
               </div>"#,
            name, cpu_str, name, mem_str, name, name
        ));
    }
    if gpus_display != r#"<span style="color: #777;">-</span>"# {
        res.push_str(&gpus_display);
    } else if !is_running {
        res.push_str(r#"<span style="color: #777;">-</span>"#);
    }
    res
}

pub fn render_autostart_cell(name: &str, autostart_enabled: bool) -> String {
    let autostart_checked = if autostart_enabled { "checked" } else { "" };
    format!(
        r#"<label class="nix-switch">
            <input type="checkbox" onchange="toggleAutostart('{}', this.checked)" {}>
            <span class="nix-slider"></span>
        </label>"#,
        name, autostart_checked
    )
}
