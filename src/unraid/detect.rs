use super::parse::parse_ini_file;

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