use std::collections::HashMap;

/// Native Unraid WebUI notification helper
pub fn send_unraid_notification(subject: &str, description: &str, importance: &str) {
    let importance_flag = match importance {
        "alert" => "alert",
        "warning" => "warning",
        _ => "normal",
    };
    let _ = std::process::Command::new("/usr/local/emhttp/webGui/scripts/notify")
        .args([
            "-e", "Nix Plugin",
            "-s", subject,
            "-d", description,
            "-i", importance_flag,
        ])
        .stdin(std::process::Stdio::null())
        .output();
}

/// Query active or stopped Docker container ports mapping
pub fn get_docker_mapped_ports() -> Vec<u16> {
    let mut ports = Vec::new();
    let output = std::process::Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Ports}}"])
        .stdin(std::process::Stdio::null())
        .output();
        
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            // e.g. "0.0.0.0:8080->80/tcp, :::8080->80/tcp"
            for part in line.split(',') {
                if let Some(arrow_idx) = part.find("->") {
                    let before_arrow = &part[..arrow_idx];
                    if let Some(colon_idx) = before_arrow.rfind(':') {
                        let port_str = &before_arrow[colon_idx + 1..];
                        if let Ok(port) = port_str.parse::<u16>() {
                            ports.push(port);
                        }
                    }
                }
            }
        }
    }
    ports
}

/// Parse generic Unraid INI configuration files
pub fn parse_ini_file(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let val = line[pos + 1..].trim().trim_matches('"').to_string();
                map.insert(key, val);
            }
        }
    }
    map
}

/// Auto-detect Unraid's system storage pool config path
pub fn detect_default_store_path() -> String {
    let system_cfg = parse_ini_file("/boot/config/shares/system.cfg");
    let mut pool = system_cfg.get("shareCachePool").cloned().unwrap_or_default();
    if pool.is_empty() {
        if let Some(use_cache) = system_cfg.get("shareUseCache") {
            if use_cache == "yes" || use_cache == "prefer" || use_cache == "only" {
                pool = "cache".to_string();
            }
        }
    }
    if !pool.is_empty() {
        let path = format!("/mnt/{}/system/nix", pool);
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/system").is_dir() {
        return "/mnt/user/system/nix".to_string();
    }
    "".to_string()
}

/// Auto-detect Unraid's AppData pool root path
pub fn detect_appdata_root() -> String {
    let appdata_cfg = parse_ini_file("/boot/config/shares/appdata.cfg");
    let mut pool = appdata_cfg.get("shareCachePool").cloned().unwrap_or_default();
    if pool.is_empty() {
        if let Some(use_cache) = appdata_cfg.get("shareUseCache") {
            if use_cache == "yes" || use_cache == "prefer" || use_cache == "only" {
                pool = "cache".to_string();
            }
        }
    }
    if !pool.is_empty() {
        let path = format!("/mnt/{}/appdata", pool);
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/appdata").is_dir() {
        return "/mnt/user/appdata".to_string();
    }
    "".to_string()
}
