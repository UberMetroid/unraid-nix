use crate::config;
use serde_json::Value;
use std::process::exit;

pub fn get_metadata(name: &str) {
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
    println!("{}", get_metadata_json(name));
}

pub fn get_metadata_json(name: &str) -> String {
    let meta_file = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if let Ok(content) = std::fs::read_to_string(&meta_file) {
        if let Ok(v) = serde_json::from_str::<Value>(&content) {
            let result = serde_json::json!({
                "success": true,
                "metadata": v
            });
            return serde_json::to_string(&result).unwrap_or_default();
        }
    }

    let detected_root = crate::cli::settings::detect_appdata_root();
    let fallback_appdata_root = if !detected_root.is_empty() { &detected_root } else { "/mnt/user/appdata" };

    let mut puid = "99".to_string();
    let mut pgid = "100".to_string();
    let mut appdata = format!("{}/{}", fallback_appdata_root, name);
    let mut uri = format!("nixpkgs#{}", name);
    let gpu = "0".to_string();

    let cfg_file = "/boot/config/plugins/nix/process-compose.yml";
    if let Ok(cfg) = config::load_config(cfg_file) {
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
                    uri = format!("nixpkgs#{}", raw_uri);
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
