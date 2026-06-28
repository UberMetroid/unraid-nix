pub fn get_service_ports(name: &str) -> Vec<crate::sandbox::PortMapping> {
    let mut ports = Vec::new();
    // Validate service name to prevent path traversal before file I/O.
    if !crate::store::is_valid_service_name(name) {
        return ports;
    }
    let metadata_path = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(port_val) = val.get("port") {
                    if let Some(num) = port_val.as_u64() {
                        if num > 0 && num <= u16::MAX as u64 {
                            ports.push(crate::sandbox::PortMapping { host: num as u16, container: num as u16 });
                        }
                    }
                    if let Some(s) = port_val.as_str() {
                        let mappings = crate::sandbox::parse_ports(s);
                        if !mappings.is_empty() {
                            return mappings;
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
                    for port_item in ports_arr {
                        if let (Some(h), Some(c)) = (port_item.get("host").and_then(|v| v.as_u64()), port_item.get("container").and_then(|v| v.as_u64())) {
                            if h <= u16::MAX as u64 && c <= u16::MAX as u64 {
                                ports.push(crate::sandbox::PortMapping { host: h as u16, container: c as u16 });
                            }
                        }
                    }
                }
            }
        }
    }

    if ports.is_empty() {
        if name_lower.contains("sonarr") {
            ports.push(crate::sandbox::PortMapping { host: 8989, container: 8989 });
        } else if name_lower.contains("radarr") {
            ports.push(crate::sandbox::PortMapping { host: 7878, container: 7878 });
        } else if name_lower.contains("jellyfin") {
            ports.push(crate::sandbox::PortMapping { host: 8096, container: 8096 });
        } else if name_lower.contains("syncthing") {
            ports.push(crate::sandbox::PortMapping { host: 8384, container: 8384 });
        }
    }

    ports
}
