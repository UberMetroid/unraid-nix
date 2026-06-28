use crate::unraid::METADATA_DIR;

pub fn get_service_ports(name: &str) -> Vec<crate::sandbox::PortMapping> {
    let mut ports = Vec::new();
    if !crate::store::is_valid_service_name(name) {
        return ports;
    }
    let metadata_path = format!("{METADATA_DIR}/{name}.json");
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(port_val) = val.get("port") {
                    if let Some(num) = port_val.as_u64() {
                        if num > 0 {
                            if let Ok(p) = u16::try_from(num) {
                                ports.push(crate::sandbox::PortMapping {
                                    host: p,
                                    container: p,
                                });
                            }
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
                        if let (Some(h), Some(c)) = (
                            port_item.get("host").and_then(|v| v.as_u64()),
                            port_item.get("container").and_then(|v| v.as_u64()),
                        ) {
                            if let (Ok(host), Ok(container)) = (u16::try_from(h), u16::try_from(c))
                            {
                                ports.push(crate::sandbox::PortMapping { host, container });
                            }
                        }
                    }
                }
            }
        }
    }

    if ports.is_empty() {
        if name_lower.contains("sonarr") {
            ports.push(crate::sandbox::PortMapping {
                host: 8989,
                container: 8989,
            });
        } else if name_lower.contains("radarr") {
            ports.push(crate::sandbox::PortMapping {
                host: 7878,
                container: 7878,
            });
        } else if name_lower.contains("jellyfin") {
            ports.push(crate::sandbox::PortMapping {
                host: 8096,
                container: 8096,
            });
        } else if name_lower.contains("syncthing") {
            ports.push(crate::sandbox::PortMapping {
                host: 8384,
                container: 8384,
            });
        }
    }

    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_service_ports_invalid_name_returns_empty() {
        // Invalid service names must short-circuit before any I/O so we
        // don't try to read a malformed path or panic on validation.
        assert!(get_service_ports("../bad").is_empty());
        assert!(get_service_ports("").is_empty());
        assert!(get_service_ports(".dot").is_empty());
    }

    #[test]
    fn test_get_service_ports_falls_back_to_known_defaults() {
        // For a name that matches one of the well-known defaults and has
        // no metadata file or preset file in the test environment, we get
        // back the canonical default mapping.
        let sonarr = get_service_ports("sonarr");
        assert_eq!(
            sonarr,
            vec![crate::sandbox::PortMapping {
                host: 8989,
                container: 8989
            }]
        );

        let radarr = get_service_ports("radarr");
        assert_eq!(
            radarr,
            vec![crate::sandbox::PortMapping {
                host: 7878,
                container: 7878
            }]
        );
    }
}
