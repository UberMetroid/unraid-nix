use std::collections::HashMap;

pub const SUPERVISOR_PORT: u16 = 29704;
pub const PROCESS_COMPOSE_CONFIG: &str = "/boot/config/plugins/nix/process-compose.yml";
pub const METADATA_DIR: &str = "/boot/config/plugins/nix/metadata";
pub const NIX_CFG_PATH: &str = "/boot/config/plugins/nix/nix.cfg";

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
                if !val.is_empty() {
                    map.insert(key, val);
                }
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
        let path = format!("/mnt/{pool}/system/nix");
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/system").is_dir() {
        return "/mnt/user/system/nix".to_string();
    }
    String::new()
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
        let path = format!("/mnt/{pool}/appdata");
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/appdata").is_dir() {
        return "/mnt/user/appdata".to_string();
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_file() {
        let content = "
; This is a comment
# Another comment
key1 = value1
key2 = \"value2\"
key3=value3
";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_ini.cfg");
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("key1").unwrap(), "value1");
        assert_eq!(map.get("key2").unwrap(), "value2");
        assert_eq!(map.get("key3").unwrap(), "value3");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_parse_ini_file_empty_values_omitted() {
        // Empty values should be skipped so that consumers' default values apply.
        let content = "
present = hello
empty =
quoted_empty = \"\"
";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_empty-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("present").unwrap(), "hello");
        assert!(map.get("empty").is_none(), "empty value should be skipped");
        assert!(map.get("quoted_empty").is_none(), "quoted empty value should be skipped");

        let _ = std::fs::remove_file(file_path);
    }
}
