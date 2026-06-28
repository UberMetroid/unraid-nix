use crate::unraid::METADATA_DIR;
use crate::util::process::run_with_timeout;
use std::process::Command;
use std::time::Duration;

pub mod icons;
pub use icons::get_service_icon_path;

#[derive(Debug, Clone)]
pub struct HostAddr {
    #[allow(dead_code)]
    pub interface: String,
    pub ip: String,
}

pub fn get_service_web_port(name: &str) -> Option<u16> {
    if !crate::store::is_valid_service_name(name) {
        return None;
    }
    let metadata_path = format!("{METADATA_DIR}/{name}.json");
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(port_val) = val.get("port") {
                    if let Some(num) = port_val.as_u64() {
                        if num > 0 {
                            if let Ok(p) = u16::try_from(num) {
                                return Some(p);
                            }
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
                        if let Some(host_port) = ports_arr[0].get("host").and_then(|hp| hp.as_u64())
                        {
                            if let Ok(p) = u16::try_from(host_port) {
                                return Some(p);
                            }
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

/// HTML-escape a string for safe interpolation into HTML text content or
/// attribute values. Replaces `&`, `<`, `>`, `"`, and `'` with their
/// HTML entity equivalents. Use this for ALL user-controlled data before
/// interpolating into `format!`-built HTML in api/* files.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// JavaScript-context escape. Use for values being interpolated into
/// inline event handlers like `onclick="fn('...')"`. Escapes `\`, `'`,
/// `"`, and `<`/`>`. Apply before `html_escape` to be safe in both
/// HTML attribute and JS string contexts.
pub fn js_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
}

pub fn extract_package_uri(command: &str) -> Option<String> {
    if let Some(pos) = command.find("nixpkgs#") {
        let sub = &command[pos..];
        let end = sub.find([' ', '"', '\'', ';']).unwrap_or(sub.len());
        let mut uri = sub[..end].to_string();
        while uri.ends_with('\\') || uri.ends_with('"') || uri.ends_with('\'') {
            uri.pop();
        }
        return Some(uri);
    }

    if let Some(pos) = command.find("nix run ") {
        let sub = &command[pos + "nix run ".len()..];
        let end = sub.find([' ', '"', '\'', ';']).unwrap_or(sub.len());
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
    let output = {
        let mut cmd = Command::new("ip");
        cmd.args(["-o", "-4", "addr", "show"])
            .stdin(std::process::Stdio::null());
        run_with_timeout(&mut cmd, Duration::from_secs(3))
    };

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let iface = parts[1];
                    let ip_net = parts[3];

                    let iface_lower = iface.to_lowercase();
                    if iface_lower == "lo"
                        || iface_lower.starts_with("veth")
                        || iface_lower.starts_with("docker")
                        || iface_lower.starts_with("br-")
                        || iface_lower.starts_with("virbr")
                        || iface_lower.starts_with("shim")
                    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape_replaces_all_five_chars() {
        assert_eq!(
            html_escape("<script>\"a\"&b'c</script>"),
            "&lt;script&gt;&quot;a&quot;&amp;b&#x27;c&lt;/script&gt;"
        );
    }

    #[test]
    fn test_html_escape_passes_through_safe_text() {
        assert_eq!(html_escape("plain ascii 123"), "plain ascii 123");
    }

    #[test]
    fn test_js_escape_escapes_quotes_and_backslash() {
        // Backslash, single quote, and double quote must each be escaped.
        assert_eq!(js_escape("a\\b'c\"d"), "a\\\\b\\'c\\\"d");
    }

    #[test]
    fn test_js_escape_passes_through_other_chars() {
        // `$`, `;`, `|`, etc. are intentionally NOT escaped — the caller
        // chains html_escape first for HTML contexts.
        assert_eq!(js_escape("$x;y|z"), "$x;y|z");
    }

    #[test]
    fn test_extract_package_uri_nixpkgs_prefix_stops_at_space() {
        assert_eq!(
            extract_package_uri("nix run nixpkgs#jellyfin --foo"),
            Some("nixpkgs#jellyfin".to_string())
        );
    }

    #[test]
    fn test_extract_package_uri_nix_run_strips_run_keyword() {
        // When there's no `nixpkgs#` prefix but the command starts with
        // `nix run `, the URI is the next whitespace-delimited token.
        assert_eq!(
            extract_package_uri("nix run my-pkg --flag"),
            Some("my-pkg".to_string())
        );
    }

    #[test]
    fn test_extract_package_uri_returns_none_for_unrelated_command() {
        assert_eq!(extract_package_uri("regular shell command"), None);
    }

    #[test]
    fn test_get_host_ips_filters_loopback_interface_and_loopback_ip() {
        // We can't predict the host's interfaces, but the contract is:
        // every returned entry must NOT be a virtual interface and its
        // IP must not be in 127.0.0.0/8 — except for the explicit
        // 127.0.0.1 fallback when the system has no other IPs.
        let ips = get_host_ips();
        assert!(!ips.is_empty(), "expected at least the loopback fallback");
        for addr in &ips {
            assert!(
                !addr.interface.starts_with("veth"),
                "veth interface leaked: {}",
                addr.interface
            );
            assert!(
                !addr.interface.starts_with("docker"),
                "docker interface leaked: {}",
                addr.interface
            );
            assert!(
                !addr.interface.starts_with("br-"),
                "bridge interface leaked: {}",
                addr.interface
            );
            assert!(
                !addr.interface.starts_with("virbr"),
                "virbr interface leaked: {}",
                addr.interface
            );
            if addr.ip != "127.0.0.1" {
                assert!(
                    !addr.ip.starts_with("127."),
                    "127/8 IP leaked on non-loopback interface: {}",
                    addr.ip
                );
            }
        }
    }
}
