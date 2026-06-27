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

pub fn get_service_appdata_path(name: &str, command: &str) -> String {
    let metadata_path = format!("/boot/config/plugins/nix/metadata/{}.json", name);
    if let Ok(content) = std::fs::read_to_string(&metadata_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(appdata_val) = val.get("appdata").and_then(|a| a.as_str()) {
                if !appdata_val.trim().is_empty() {
                    return appdata_val.to_string();
                }
            }
        }
    }

    if let Some(pos) = command.find("mount --bind ") {
        let sub = &command[pos + "mount --bind ".len()..];
        if let Some(end_pos) = sub.find(" /config") {
            let path = sub[..end_pos].trim();
            if !path.is_empty() {
                return path.to_string();
            }
        }
    }

    extract_home_path(command)
}

pub fn extract_package_uri(command: &str) -> Option<String> {
    if let Some(pos) = command.find("nixpkgs#") {
        let sub = &command[pos..];
        let end = sub.find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ';')
            .unwrap_or(sub.len());
        let mut uri = sub[..end].to_string();
        while uri.ends_with('\\') || uri.ends_with('"') || uri.ends_with('\'') {
            uri.pop();
        }
        return Some(uri);
    }
    
    if let Some(pos) = command.find("nix run ") {
        let sub = &command[pos + "nix run ".len()..];
        let end = sub.find(|c: char| c == ' ' || c == '"' || c == '\'' || c == ';')
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

pub fn get_service_icon_path(name: &str) -> Option<String> {
    let config_path = "/boot/config/plugins/nix/process-compose.yml";
    let config = crate::config::load_config(config_path).ok()?;
    let p = config.processes.get(name)?;
    let command = &p.command;

    if let Some(idx) = command.find("/nix/store/") {
        let sub = &command[idx..];
        let parts: Vec<&str> = sub.split('/').collect();
        if parts.len() >= 4 {
            let candidate = format!("/{}/{}/{}", parts[1], parts[2], parts[3]);
            if !candidate.contains("-bash-") && !candidate.contains("-bash-interactive-") && candidate.contains(name) {
                let path = std::path::Path::new(&candidate);
                if path.exists() {
                    if let Some(icon) = find_image_in_dir(path, path) {
                        return Some(icon);
                    }
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("/nix/store") {
        let target_pattern = format!("-{}", name.to_lowercase());
        let mut candidates = Vec::new();
        
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy().to_lowercase();
            if !filename_str.ends_with(".drv") && filename_str.contains(&target_pattern) {
                candidates.push(entry.path());
            }
        }

        candidates.sort_by_key(|path| {
            let name_str = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let has_extra = name_str.contains("-web") || name_str.contains("-ffmpeg") || name_str.contains("-data") || name_str.contains("-bin") || name_str.contains("-lib");
            if has_extra { 1 } else { 0 }
        });

        for cand_path in candidates {
            if cand_path.exists() {
                if let Some(icon) = find_image_in_dir(&cand_path, &cand_path) {
                    return Some(icon);
                }
            }
        }
    }

    None
}

fn find_image_in_dir(dir: &std::path::Path, root: &std::path::Path) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(rel_path) = path.strip_prefix(root) {
                if rel_path == std::path::Path::new("bin") || 
                   rel_path == std::path::Path::new("man") || 
                   rel_path == std::path::Path::new("nix-support") ||
                   rel_path == std::path::Path::new("lib64") {
                    continue;
                }
            }
            subdirs.push(path);
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "svg" || ext_lower == "png" || ext_lower == "ico" {
                    candidates.push(path);
                }
            }
        }
    }

    for c in &candidates {
        if let Some(filename) = c.file_name().and_then(|f| f.to_str()) {
            let name_lower = filename.to_lowercase();
            if name_lower == "logo.svg" || name_lower == "logo.png" || name_lower.contains("jellyfin.svg") {
                return Some(c.to_string_lossy().to_string());
            }
        }
    }

    for c in &candidates {
        if let Some(filename) = c.file_name().and_then(|f| f.to_str()) {
            let name_lower = filename.to_lowercase();
            if name_lower.contains("logo") || name_lower.contains("icon") || name_lower.contains("favicon") {
                return Some(c.to_string_lossy().to_string());
            }
        }
    }

    for subdir in subdirs {
        if let Some(img) = find_image_in_dir(&subdir, root) {
            return Some(img);
        }
    }
    None
}
