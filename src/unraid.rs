use std::collections::HashMap;
use std::time::Duration;

use crate::util::process::run_with_timeout;

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
    let _ = {
        let mut cmd = std::process::Command::new("/usr/local/emhttp/webGui/scripts/notify");
        cmd.args([
            "-e", "Nix Plugin",
            "-s", subject,
            "-d", description,
            "-i", importance_flag,
        ])
        .stdin(std::process::Stdio::null());
        run_with_timeout(&mut cmd, Duration::from_secs(5))
    };
}

/// Query active or stopped Docker container ports mapping
pub fn get_docker_mapped_ports() -> Vec<u16> {
    let mut ports = Vec::new();
    let output = {
        let mut cmd = std::process::Command::new("docker");
        cmd.args(["ps", "-a", "--format", "{{.Ports}}"])
            .stdin(std::process::Stdio::null());
        run_with_timeout(&mut cmd, Duration::from_secs(5))
    };

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
        assert!(!map.contains_key("empty"), "empty value should be skipped");
        assert!(!map.contains_key("quoted_empty"), "quoted empty value should be skipped");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_parse_ini_file_trims_surrounding_whitespace_around_keys_and_values() {
        // The parser trims the LHS (key) and the RHS (value, before
        // stripping surrounding quotes). Verify both directions so a
        // future refactor doesn't accidentally drop one.
        let content = "  spaced_key  =  spaced value  \n";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_ws-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("spaced_key").unwrap(), "spaced value");
        assert!(map.contains_key("spaced_key"));

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_parse_ini_file_preserves_internal_whitespace_in_value() {
        // The parser must not trim internal whitespace — only the
        // boundaries around key and value.
        let content = "phrase = hello   world\n";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_internal_ws-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("phrase").unwrap(), "hello   world");

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_parse_ini_file_handles_line_without_equals() {
        // A line without an `=` is silently skipped (not an error). This
        // matches the legacy parser behavior.
        let content = "just-a-line-no-equals\nreal_key = value\n";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_no_eq-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert!(!map.contains_key("just-a-line-no-equals"));
        assert_eq!(map.get("real_key").unwrap(), "value");

        let _ = std::fs::remove_file(&file_path);
    }

    #[test]
    fn test_parse_ini_file_handles_value_with_embedded_equals() {
        // `find('=')` returns the FIRST `=`, so any `=` inside the value
        // is preserved. Verify this contract.
        let content = "url = http://example.com/path?a=1&b=2\n";
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "test_ini_eq_in_val-{}.cfg",
            std::process::id()
        ));
        std::fs::write(&file_path, content).unwrap();

        let map = parse_ini_file(file_path.to_str().unwrap());
        assert_eq!(map.get("url").unwrap(), "http://example.com/path?a=1&b=2");

        let _ = std::fs::remove_file(&file_path);
    }
}
