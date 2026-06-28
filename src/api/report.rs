use crate::api::utils::html_escape;
use crate::unraid::METADATA_DIR;
use serde_json::Value;
use std::fs;

pub fn render_verification_report(service: &str) -> String {
    if !crate::store::is_valid_service_name(service) {
        return String::new();
    }
    let metadata_file = format!("{METADATA_DIR}/{service}.json");
    let content = match fs::read_to_string(&metadata_file) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let meta: Value = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };

    let name = meta.get("name").and_then(|v| v.as_str()).unwrap_or(service);
    let uri = meta.get("uri").and_then(|v| v.as_str()).unwrap_or("");

    let puid = meta
        .get("puid")
        .and_then(|v| v.as_u64())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            meta.get("puid")
                .and_then(|v| v.as_str())
                .unwrap_or("99")
                .to_string()
        });

    let pgid = meta
        .get("pgid")
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
    let gpus = meta.get("gpus").and_then(|v| v.as_str()).unwrap_or("");

    let puid_label = if puid == "99" {
        "nobody (99)".to_string()
    } else {
        html_escape(&puid)
    };
    let pgid_label = if pgid == "100" {
        "users (100)".to_string()
    } else {
        html_escape(&pgid)
    };

    let gpu_label = if !gpus.trim().is_empty() {
        format!("Enabled (Exposing GPU: {})", html_escape(gpus))
    } else if is_gpu {
        "Enabled (Exposing All GPUs)".to_string()
    } else {
        "Disabled".to_string()
    };
    let gpu_icon = if !gpus.trim().is_empty() || is_gpu {
        "fa-check success"
    } else {
        "fa-minus-circle warning"
    };

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
                if let (Some(host), Some(sandbox)) = (
                    item.get("host").and_then(|v| v.as_str()),
                    item.get("sandbox").and_then(|v| v.as_str()),
                ) {
                    extra_binds_html.push_str(&format!(
                        "<div style='margin-bottom:4px;'><i class='fa fa-check success'></i> {} &rarr; {}</div>",
                        html_escape(host), html_escape(sandbox)
                    ));
                }
            }
        }
    }
    if extra_binds_html.is_empty() {
        extra_binds_html =
            "<div><i class='fa fa-check success'></i> None configured</div>".to_string();
    }

    let mut ports_html = String::new();
    if let Some(port_val) = meta.get("port").and_then(|v| v.as_str()) {
        if !port_val.trim().is_empty() {
            for p in port_val.split(',') {
                if !p.trim().is_empty() {
                    ports_html.push_str(&format!(
                        "<div style='margin-bottom:4px;'><i class='fa fa-check success'></i> Host Port Mapped: {}</div>",
                        html_escape(p.trim())
                    ));
                }
            }
        }
    }
    if ports_html.is_empty() {
        ports_html = "<div><i class='fa fa-check success'></i> Running on default ports (no overrides)</div>".to_string();
    }

    let mut env_vars_html = String::new();
    if let Some(env_vars_val) = meta.get("env_vars") {
        let envs_list = if env_vars_val.is_string() {
            let s = env_vars_val.as_str().unwrap_or("");
            serde_json::from_str::<Value>(s).ok()
        } else {
            Some(env_vars_val.clone())
        };

        if let Some(Value::Object(map)) = envs_list {
            for (k, v) in map {
                let v_str = if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                };
                env_vars_html.push_str(&format!(
                    "<div style='margin-bottom:4px;'><i class='fa fa-check success'></i> <code>{}</code> = <code>{}</code></div>",
                    html_escape(&k), html_escape(&v_str)
                ));
            }
        }
    }
    if env_vars_html.is_empty() {
        env_vars_html =
            "<div><i class='fa fa-check success'></i> None configured</div>".to_string();
    }

    let sandbox_desc = if crate::sandbox::is_storage_sandbox_enabled() {
        format!(
            "Private mount namespace (unshare -m) chroot jail at /var/run/nix-chroot-{}",
            html_escape(name)
        )
    } else {
        "Disabled (running directly in host mount namespace)".to_string()
    };

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
                <div><strong>GPU Passthrough:</strong></div><div><i class='fa {}'></i> {}</div>
                <div><strong>Storage Sandboxing:</strong></div><div><i class='fa fa-check success'></i> {}</div>
                <div><strong>Appdata Bind-Mount:</strong></div><div><i class='fa fa-check success'></i> {} &rarr; /config</div>
            </div>
            <div style='display: grid; grid-template-columns: 180px 1fr; gap: 8px 12px; font-size: 12px; color: #eee;'>
                <div><strong>Network Mappings:</strong></div><div>{}</div>
                <div><strong>Environment Variables:</strong></div><div>{}</div>
                <div><strong>Shared Storage Paths:</strong></div><div>{}</div>
            </div>
        </div>"#,
        html_escape(name),
        html_escape(uri),
        puid_label,
        pgid_label,
        gpu_icon,
        gpu_label,
        sandbox_desc,
        html_escape(appdata),
        ports_html,
        env_vars_html,
        extra_binds_html
    )
}
