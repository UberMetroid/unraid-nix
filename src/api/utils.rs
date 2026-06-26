use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct HostAddr {
    pub interface: String,
    pub ip: String,
}

pub fn get_service_web_port(name: &str) -> Option<u16> {
    let metadata_path = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(port_val) = val.get("port") {
                    if let Some(num) = port_val.as_u64() {
                        if num > 0 {
                            return Some(num as u16);
                        }
                    }
                    if let Some(s) = port_val.as_str() {
                        if let Ok(num) = s.parse::<u16>() {
                            return Some(num);
                        }
                        let mappings = crate::sandbox::parse_ports(s);
                        if !mappings.is_empty() {
                            let name_lower = name.to_lowercase();
                            if name_lower.contains("jellyfin") {
                                if let Some(m) = mappings.iter().find(|m| m.container == 8096) {
                                    return Some(m.host);
                                }
                            } else if name_lower.contains("syncthing") {
                                if let Some(m) = mappings.iter().find(|m| m.container == 8384) {
                                    return Some(m.host);
                                }
                            }
                            return Some(mappings[0].host);
                        }
                    }
                }
            }
        }
    }

    let name_lower = name.to_lowercase();
    if name_lower.contains("sonarr") {
        Some(8989)
    } else if name_lower.contains("radarr") {
        Some(7878)
    } else if name_lower.contains("jellyfin") {
        Some(8096)
    } else if name_lower.contains("syncthing") {
        Some(8384)
    } else {
        None
    }
}

pub fn extract_home_path(command: &str) -> String {
    if let Some(pos) = command.find("export HOME=") {
        let start = pos + "export HOME=".len();
        let sub = &command[start..];
        
        let mut end = sub.len();
        for (i, c) in sub.char_indices() {
            if c == ' ' || c == ';' || c == '&' || c == '"' || c == '\'' {
                end = i;
                break;
            }
        }
        let path = sub[..end].trim();
        if !path.is_empty() {
            return path.to_string();
        }
    }
    "-".to_string()
}

pub fn extract_package_uri(command: &str) -> Option<String> {
    if let Some(pos) = command.find("nixpkgs#") {
        let sub = &command[pos..];
        let end = sub.find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ';')
            .unwrap_or(sub.len());
        return Some(sub[..end].to_string());
    }
    
    if let Some(pos) = command.find("nix run ") {
        let sub = &command[pos + "nix run ".len()..];
        let end = sub.find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ';')
            .unwrap_or(sub.len());
        let uri = sub[..end].trim();
        if !uri.is_empty() {
            return Some(uri.to_string());
        }
    }
    None
}

pub fn resolve_package_version(uri: &str) -> String {
    let cmd = format!(
        ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix eval --raw {}.version 2>/dev/null",
        uri
    );
    let output = Command::new("sh")
        .args(&["-c", &cmd])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !ver.is_empty() {
                return ver;
            }
        }
    }

    let cmd_name = format!(
        ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix eval --raw {}.name 2>/dev/null",
        uri
    );
    let output_name = Command::new("sh")
        .args(&["-c", &cmd_name])
        .output();
        
    if let Ok(out) = output_name {
        if out.status.success() {
            let name_ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some(pos) = name_ver.rfind('-') {
                let ver = &name_ver[pos + 1..];
                if !ver.is_empty() && ver.chars().next().unwrap().is_digit(10) {
                    return ver.to_string();
                }
            }
            if !name_ver.is_empty() {
                return name_ver;
            }
        }
    }
    
    "unknown".to_string()
}

pub fn get_cached_version(uri: &str) -> String {
    let cache_path = "/boot/config/plugins/nix/.version_cache.json";
    let mut cache: HashMap<String, String> = std::fs::read_to_string(cache_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default();

    if let Some(ver) = cache.get(uri) {
        if ver != "unknown" {
            return ver.clone();
        }
    }

    let ver = resolve_package_version(uri);
    if ver != "unknown" {
        cache.insert(uri.to_string(), ver.clone());
        if let Ok(content) = serde_json::to_string(&cache) {
            let _ = std::fs::write(cache_path, content);
        }
    }
    ver
}

pub fn get_package_link_url(uri: &str) -> Option<String> {
    if uri.starts_with("nixpkgs#") {
        let short_name = uri.replace("nixpkgs#", "");
        return Some(format!(
            "https://search.nixos.org/packages?channel=unstable&query={}",
            short_name
        ));
    }
    if uri.starts_with("github:") {
        let clean_uri = uri.replace("github:", "");
        let base_path = clean_uri.split('#').next().unwrap_or("");
        let parts: Vec<&str> = base_path.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("https://github.com/{}/{}", parts[0], parts[1]));
        }
    }
    None
}

pub fn get_host_ips() -> Vec<HostAddr> {
    let mut ips = Vec::new();
    let output = Command::new("ip")
        .args(&["-o", "-4", "addr", "show"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let iface = parts[1];
                    let ip_net = parts[3];
                    
                    let iface_lower = iface.to_lowercase();
                    if iface_lower == "lo" || 
                       iface_lower.starts_with("veth") || 
                       iface_lower.starts_with("docker") || 
                       iface_lower.starts_with("br-") || 
                       iface_lower.starts_with("virbr") ||
                       iface_lower.starts_with("shim") {
                        continue;
                    }

                    if let Some(pos) = ip_net.find('/') {
                        let ip = &ip_net[..pos];
                        if !ip.starts_with("127.") {
                            ips.push(HostAddr {
                                interface: iface.to_string(),
                                ip: ip.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push(HostAddr {
            interface: "lo".to_string(),
            ip: "127.0.0.1".to_string(),
        });
    }
    ips
}

pub fn is_cli_enabled() -> bool {
    if let Ok(content) = std::fs::read_to_string("/boot/config/plugins/nix/nix.cfg") {
        for line in content.lines() {
            if line.starts_with("ENABLE_CLI_INSTALL=") {
                let val = line.trim_start_matches("ENABLE_CLI_INSTALL=").trim_matches('"');
                return val == "yes";
            }
        }
    }
    false
}
