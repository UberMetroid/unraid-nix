use crate::unraid::{parse_ini_file, NIX_CFG_PATH};

/// Validation check for the persistent store path.
pub fn validate_store_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Nix store path cannot be empty.".to_string());
    }
    if path.starts_with("/boot") {
        return Err("Nix store path cannot be located on the boot flash drive (/boot).".to_string());
    }
    Ok(())
}

pub fn read_cfg_val(key: &str, default: &str) -> String {
    let map = parse_ini_file(NIX_CFG_PATH);
    map.get(key).cloned().unwrap_or_else(|| default.to_string())
}

pub fn read_allow_source_builds() -> bool {
    read_cfg_val("ALLOW_SOURCE_BUILDS", "no") == "yes"
}