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
    let check_port = if let Some(p_str) = port.as_deref() {
        p_str.parse::<u16>().ok()
    } else {
        let name_lower = name.to_lowercase();
        let preset_path = crate::config::get_preset_path(&name_lower);
        std::path::Path::new(&preset_path).exists()
            .then(|| {
                std::fs::read_to_string(&preset_path).ok()
                    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                    .and_then(|json| json.get("default_ports")?.as_array().cloned())
                    .and_then(|ports_arr| ports_arr.first().cloned())
                    .and_then(|first| first.get("host")?.as_u64())
                    .and_then(|n| u16::try_from(n).ok())
            })
            .flatten()
    };

    if let Some(p) = check_port {
        if crate::process::ports::is_port_in_use(p) {
            println!("[WARNING] Port {p} is already bound by another service or Docker container on the host. This service may fail to start unless you configure a custom Port Override.");
        }
    }
}
