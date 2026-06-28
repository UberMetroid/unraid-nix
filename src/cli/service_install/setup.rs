use super::ExtraBind;

pub fn setup_appdata_dir(appdata: &str, puid: u32, pgid: u32) {
    if !appdata.is_empty() {
        let path = std::path::Path::new(appdata);
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
            #[cfg(unix)]
            let _ = std::os::unix::fs::chown(path, Some(puid), Some(pgid));
        }
    }
}

pub fn parse_and_create_binds(extra_binds_json: &str, puid: u32, pgid: u32) -> Vec<ExtraBind> {
    let extra_binds: Vec<ExtraBind> = if !extra_binds_json.is_empty() {
        serde_json::from_str(extra_binds_json).unwrap_or_default()
    } else {
        Vec::new()
    };

    for b in &extra_binds {
        let host_path = b.host.trim();
        if !host_path.is_empty() {
            let path = std::path::Path::new(host_path);
            if !path.exists() {
                let _ = std::fs::create_dir_all(path);
                #[cfg(unix)]
                let _ = std::os::unix::fs::chown(path, Some(puid), Some(pgid));
            }
        }
    }
    extra_binds
}

pub fn verify_port_conflicts(name: &str, port: &Option<String>) {
    let check_port = if let Some(ref p_str) = port {
        p_str.parse::<u16>().ok()
    } else {
        let name_lower = name.to_lowercase();
        let preset_path = crate::config::get_preset_path(&name_lower);
        if std::path::Path::new(&preset_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&preset_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ports_arr) = json.get("default_ports").and_then(|p| p.as_array()) {
                        if !ports_arr.is_empty() {
                            ports_arr[0].get("host").and_then(|hp| hp.as_u64()).map(|p| p as u16)
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    };

    if let Some(p) = check_port {
        if crate::process::ports::is_port_in_use(p) {
            println!("[WARNING] Port {} is already bound by another service or Docker container on the host. This service may fail to start unless you configure a custom Port Override.", p);
        }
    }
}
