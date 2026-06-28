use std::collections::HashMap;

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
                map.insert(key, val);
            }
        }
    }
    map
}

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
        let path = format!("/mnt/{}/system/nix", pool);
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/system").is_dir() {
        return "/mnt/user/system/nix".to_string();
    }
    "".to_string()
}

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
        let path = format!("/mnt/{}/appdata", pool);
        if std::path::Path::new(&path).is_dir() {
            return path;
        }
    }
    if std::path::Path::new("/mnt/user/appdata").is_dir() {
        return "/mnt/user/appdata".to_string();
    }
    "".to_string()
}

pub fn has_files(dir: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        entries.count() > 0
    } else {
        false
    }
}
