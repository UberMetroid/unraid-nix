use std::collections::HashMap;
use std::process::Command;

const NIX_BIN: &str = "/nix/var/nix/profiles/default/bin/nix";

fn is_valid_uri(uri: &str) -> bool {
    if uri.is_empty() {
        return false;
    }
    if uri.contains("..") {
        return false;
    }
    if uri.starts_with("nixpkgs#") {
        let attr = &uri["nixpkgs#".len()..];
        return !attr.is_empty()
            && attr
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-'));
    }
    if let Some(rest) = uri.strip_prefix("github:") {
        if let Some((owner_repo, attr)) = rest.split_once('#') {
            let parts: Vec<&str> = owner_repo.split('/').collect();
            let valid_parts = parts.len() == 2
                && !parts[0].is_empty()
                && !parts[1].is_empty()
                && parts[0]
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
                && parts[1]
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
            let valid_attr = !attr.is_empty()
                && attr
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-'));
            return valid_parts && valid_attr;
        }
        let parts: Vec<&str> = rest.split('/').collect();
        return parts.len() == 2
            && !parts[0].is_empty()
            && !parts[1].is_empty()
            && parts[0]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            && parts[1]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    }
    if let Some(rest) = uri.strip_prefix("path:") {
        return rest.starts_with('/')
            && !rest.contains('\0')
            && rest
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '.' | '-'));
    }
    uri.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '#' | ':' | '+' | '@' | '/' | '-'))
}

pub fn resolve_package_version(uri: &str) -> String {
    if !is_valid_uri(uri) {
        return "unknown".to_string();
    }

    let output = Command::new(NIX_BIN)
        .arg("eval")
        .arg("--raw")
        .arg(format!("{}.version", uri))
        .stdin(std::process::Stdio::null())
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !ver.is_empty() {
                return ver;
            }
        }
    }

    let output_name = Command::new(NIX_BIN)
        .arg("eval")
        .arg("--raw")
        .arg(format!("{}.name", uri))
        .stdin(std::process::Stdio::null())
        .output();

    if let Ok(out) = output_name {
        if out.status.success() {
            let name_ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some(pos) = name_ver.rfind('-') {
                let ver = &name_ver[pos + 1..];
                if !ver.is_empty() && ver.starts_with(|c: char| c.is_ascii_digit()) {
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
