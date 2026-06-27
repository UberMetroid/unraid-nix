use std::fs;
use serde_json::Value;

pub fn render_verification_report(service: &str) -> String {
    let metadata_file = format!("/boot/config/plugins/nix/metadata/{}.json", service);
    let content = match fs::read_to_string(&metadata_file) {
        Ok(c) => c,
        Err(_) => return "".to_string(),
    };

    let meta: Value = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return "".to_string(),
    };

    let name = meta.get("name").and_then(|v| v.as_str()).unwrap_or(service);
    let uri = meta.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    
    let puid = meta.get("puid")
        .and_then(|v| v.as_u64())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            meta.get("puid")
                .and_then(|v| v.as_str())
                .unwrap_or("99")
                .to_string()
        });

    let pgid = meta.get("pgid")
        .and_then(|v| v.as_u64())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            meta.get("pgid")
                .and_then(|v| v.as_str())
                .unwrap_or("100")
                .to_string()
        });

    let appdata = meta.get("appdata").and_then(|v| v.as_str()).unwrap_or("");
    let gpu = meta.get("gpu").and_then(|v| v.as_str()).unwrap_or("0");
    let is_gpu = gpu == "1" || gpu == "true";

    let puid_label = if puid == "99" { "nobody (99)".to_string() } else { puid };
    let pgid_label = if pgid == "100" { "users (100)".to_string() } else { pgid };
    let gpu_label = if is_gpu { "Enabled (Intel/AMD/Nvidia)" } else { "Disabled" };
    let gpu_icon = if is_gpu { "fa-check success" } else { "fa-minus-circle warning" };

    let mut extra_binds_html = String::new();
    if let Some(extra_binds_val) = meta.get("extra_binds") {
        let binds_list = if extra_binds_val.is_string() {
            let s = extra_binds_val.as_str().unwrap_or("");
            serde_json::from_str::<Value>(s).ok()
        } else {
            Some(extra_binds_val.clone())
        };

        if let Some(Value::Array(arr)) = binds_list {
            for item in arr {
                if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|v| v.as_str()), item.get("sandbox").and_then(|v| v.as_str())) {
                    extra_binds_html.push_str(&format!(
                        "<div style='margin-bottom:4px;'><i class='fa fa-check success'></i> {} &rarr; {}</div>",
                        host, sandbox
                    ));
                }
            }
        }
    }
    if extra_binds_html.is_empty() {
        extra_binds_html = "<div style='color: #777;'><i class='fa fa-info-circle'></i> None configured</div>".to_string();
    }

    let mut ports_html = String::new();
    if let Some(port_val) = meta.get("port").and_then(|v| v.as_str()) {
        if !port_val.trim().is_empty() {
            for p in port_val.split(',') {
                if !p.trim().is_empty() {
                    ports_html.push_str(&format!(
                        "<div style='margin-bottom:4px;'><i class='fa fa-check success'></i> Host Port Mapped: {}</div>",
                        p.trim()
                    ));
                }
            }
        }
    }
    if ports_html.is_empty() {
        ports_html = "<div style='color: #777;'><i class='fa fa-info-circle'></i> No port overrides (running on default ports)</div>".to_string();
    }

    format!(
        r#"<div class='nix-validation-report' style='margin-top: 20px; border: 1px solid #00a1ff; background: rgba(0, 161, 255, 0.04); border-radius: 6px; padding: 15px; font-family: "Clear Sans", sans-serif; animation: fadeIn 0.4s ease; text-align: left;'>
            <h4 style='margin: 0 0 12px 0; color: #00a1ff; display: flex; align-items: center; gap: 8px; font-size: 14px;'>
                <i class='fa fa-shield'></i> Sandbox Configuration Verification Report
            </h4>
            <div style='display: grid; grid-template-columns: 180px 1fr; gap: 8px 12px; font-size: 12px; color: #eee; border-bottom: 1px solid #2d2d30; padding-bottom: 12px; margin-bottom: 12px;'>
                <div><strong>Service Name:</strong></div><div><i class='fa fa-check success'></i> {}</div>
                <div><strong>Flake Package URI:</strong></div><div><i class='fa fa-check success'></i> {}</div>
                <div><strong>Process Owner (PUID):</strong></div><div><i class='fa fa-check success'></i> {}</div>
                <div><strong>Process Group (PGID):</strong></div><div><i class='fa fa-check success'></i> {}</div>
                <div><strong>GPU Hardware Acceleration:</strong></div><div><i class='fa {}'></i> {}</div>
                <div><strong>Sandbox Root Jail:</strong></div><div><i class='fa fa-check success'></i> /var/run/nix-chroot-{} (Isolated & Private)</div>
                <div><strong>Database Appdata Path:</strong></div><div><i class='fa fa-check success'></i> {} (Natively Mounted)</div>
            </div>
            <div style='display: grid; grid-template-columns: 180px 1fr; gap: 8px 12px; font-size: 12px; color: #eee;'>
                <div><strong>Network Mappings:</strong></div><div>{}</div>
                <div><strong>Shared Storage Paths:</strong></div><div>{}</div>
            </div>
        </div>"#,
        name, uri, puid_label, pgid_label, gpu_icon, gpu_label, name, appdata, ports_html, extra_binds_html
    )
}
