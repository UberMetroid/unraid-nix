use std::process::Command;

pub mod icons;
pub use icons::get_service_icon_path;

#[derive(Debug, Clone)]
pub struct HostAddr {
    #[allow(dead_code)]
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
    let preset_path = crate::config::get_preset_path(&name_lower);
    if std::path::Path::new(&preset_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&preset_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ports_arr) = json.get("default_ports").and_then(|p| p.as_array()) {
                    if !ports_arr.is_empty() {
                        if let Some(host_port) = ports_arr[0].get("host").and_then(|hp| hp.as_u64()) {
                            return Some(host_port as u16);
                        }
                    }
                }
            }
        }
    }

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


pub fn extract_package_uri(command: &str) -> Option<String> {
    if let Some(pos) = command.find("nixpkgs#") {
        let sub = &command[pos..];
        let end = sub.find([' ', '"', '\'', ';'])
            .unwrap_or(sub.len());
        let mut uri = sub[..end].to_string();
        while uri.ends_with('\\') || uri.ends_with('"') || uri.ends_with('\'') {
            uri.pop();
        }
        return Some(uri);
    }
    
    if let Some(pos) = command.find("nix run ") {
        let sub = &command[pos + "nix run ".len()..];
        let end = sub.find([' ', '"', '\'', ';'])
            .unwrap_or(sub.len());
        let mut uri = sub[..end].trim().to_string();
        while uri.ends_with('\\') || uri.ends_with('"') || uri.ends_with('\'') {
            uri.pop();
        }
        if !uri.is_empty() {
            return Some(uri);
        }
    }
    None
}

pub fn get_host_ips() -> Vec<HostAddr> {
    let mut ips = Vec::new();
    let output = Command::new("ip")
        .args(["-o", "-4", "addr", "show"])
        .stdin(std::process::Stdio::null())
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
