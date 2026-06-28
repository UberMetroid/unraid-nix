use crate::config;
use crate::unraid::{METADATA_DIR, NIX_CFG_PATH, PROCESS_COMPOSE_CONFIG};
use serde_json::Value;
use std::process::exit;

pub fn get_metadata(name: &str) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
    println!("{}", get_metadata_json(name));
}

pub fn get_metadata_json(name: &str) -> String {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
    let meta_file = format!("{METADATA_DIR}/{name}.json");
    if let Ok(content) = std::fs::read_to_string(&meta_file) {
        if let Ok(v) = serde_json::from_str::<Value>(&content) {
            let result = serde_json::json!({
                "success": true,
                "metadata": v
            });
            return serde_json::to_string(&result).unwrap_or_default();
        }
    }

    let mut detected_root = String::new();
    if let Ok(content) = std::fs::read_to_string(NIX_CFG_PATH) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("DEFAULT_APPDATA_PATH=") {
                if let Some(pos) = line.find('=') {
                    detected_root = line[pos + 1..].trim().trim_matches('"').to_string();
                }
            }
        }
    }
    if detected_root.is_empty() {
        detected_root = crate::unraid::detect_appdata_root();
    }
    let fallback_appdata_root = if !detected_root.is_empty() { detected_root } else { "/mnt/user/appdata".to_string() };

    let mut puid = "99".to_string();
    let mut pgid = "100".to_string();
    let mut appdata = format!("{fallback_appdata_root}/{name}");
    let mut uri = format!("nixpkgs#{name}");
    let gpu = "0".to_string();

    if let Ok(cfg) = config::load_config(PROCESS_COMPOSE_CONFIG) {
        if let Some(proc) = cfg.processes.get(name) {
            let cmd = &proc.command;
            if let Some(pos) = cmd.find("--reuid=") {
                let s = &cmd[pos + 8..];
                let end = s.find(|c: char| !c.is_numeric()).unwrap_or(s.len());
                puid = s[..end].to_string();
            }
            if let Some(pos) = cmd.find("--regid=") {
                let s = &cmd[pos + 8..];
                let end = s.find(|c: char| !c.is_numeric()).unwrap_or(s.len());
                pgid = s[..end].to_string();
            }
            if let Some(pos) = cmd.find("export HOME=") {
                let s = &cmd[pos + 12..];
                let end = s.find([' ', '&', ';', '"']).unwrap_or(s.len());
                appdata = s[..end].to_string();
            }
            if let Some(pos) = cmd.find("exec nix run ") {
                let s = &cmd[pos + 13..];
                let end = s.find([' ', '"']).unwrap_or(s.len());
                let raw_uri = s[..end].to_string();
                if raw_uri.contains('#') || raw_uri.contains(':') {
                    uri = raw_uri;
                } else {
                    uri = format!("nixpkgs#{raw_uri}");
                }
            }
        }
    }

    let result = serde_json::json!({
        "success": true,
        "metadata": {
            "name": name,
            "uri": uri,
            "appdata": appdata,
            "puid": puid,
            "pgid": pgid,
            "gpu": gpu,
            "gpus": "",
            "extra_binds": "[]",
            "port": "",
            "bind_address": ""
        }
    });

    serde_json::to_string(&result).unwrap_or_default()
}
