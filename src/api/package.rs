use std::collections::HashMap;
use std::process::Command;

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
            "https://search.nixos.org/packages?channel=unstable&show={}&query={}",
            short_name, short_name
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
