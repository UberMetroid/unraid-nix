use crate::process::is_supervisor_running;
use crate::process::ports::is_port_in_use;
use crate::unraid::METADATA_DIR;

fn run_preflight_checks(name: &str) {
    crate::store::log_event("INFO", &format!("Running pre-flight checks for service '{name}'..."));

    let default_port = match name.to_lowercase().as_str() {
        "jellyfin" => Some(8096),
        "radarr" => Some(7878),
        "sonarr" => Some(8989),
        _ => None,
    };

    if let Some(port) = default_port {
        if is_port_in_use(port) {
            crate::store::log_event(
                "WARN",
                &format!(
                    "Port conflict warning: Port {port} (default for preset '{name}') is already bound on the host."
                ),
            );
            crate::unraid::send_unraid_notification(
                &format!("Nix: Port Conflict Warning for '{name}'"),
                &format!("Port {port} is already in use on the host. Service '{name}' may fail to start."),
                "warning",
            );
        } else {
            crate::store::log_event(
                "INFO",
                &format!(
                    "Port check passed: Port {port} (default for preset '{name}') is free."
                ),
            );
        }
    }

    let metadata_path = format!("{METADATA_DIR}/{name}.json");
    if !crate::store::is_valid_service_name(name) {
        crate::store::log_event("WARN", &format!("Skipping preflight metadata check for invalid service name '{name}'"));
        return;
    }
    crate::store::log_event("DEBUG", &format!("Checking metadata configuration at path: {metadata_path}"));
    if std::path::Path::new(&metadata_path).exists() {
        match std::fs::read_to_string(&metadata_path) {
            Ok(content) => {
                #[derive(serde::Deserialize)]
                struct LocalMetadata {
                    appdata: Option<String>,
                    puid: Option<serde_json::Value>,
                }

                match serde_json::from_str::<LocalMetadata>(&content) {
                    Ok(meta) => {
                        if let Some(ref appdata_path) = meta.appdata {
                            if !appdata_path.trim().is_empty() {
                                let path = std::path::Path::new(appdata_path);
                                crate::store::log_event("DEBUG", &format!("Checking permissions for AppData path: {appdata_path}"));
                                if path.exists() {
                                    use std::os::unix::fs::MetadataExt;
                                    match std::fs::metadata(path) {
                                        Ok(fs_meta) => {
                                            let owner_uid = fs_meta.uid();
                                            let expected_puid = if let Some(ref val) = meta.puid {
                                                if let Some(num) = val.as_u64() {
                                                    u32::try_from(num).unwrap_or(99)
                                                } else if let Some(s) = val.as_str() {
                                                    s.parse::<u32>().unwrap_or(99)
                                                } else {
                                                    99
                                                }
                                            } else {
                                                99
                                            };

                                            if owner_uid != expected_puid {
                                                crate::store::log_event(
                                                    "WARN",
                                                    &format!(
                                                        "Directory permissions warning: Service '{name}' configuration location '{appdata_path}' owner UID ({owner_uid}) does not match configured PUID ({expected_puid})."
                                                     ),
                                                );
                                                crate::unraid::send_unraid_notification(
                                                    &format!("Nix: Permissions Warning for '{name}'"),
                                                    &format!("AppData directory '{appdata_path}' is owned by UID {owner_uid}, but service runs as PUID {expected_puid}."),
                                                    "warning",
                                                );
                                            } else {
                                                crate::store::log_event(
                                                    "INFO",
                                                    &format!(
                                                        "Directory permissions check passed: Service '{name}' configuration location '{appdata_path}' is owned by UID {owner_uid}."
                                                     ),
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            crate::store::log_event(
                                                "WARN",
                                                &format!(
                                                    "Failed to read file metadata for configuration location '{appdata_path}': {e}"
                                                ),
                                            );
                                        }
                                    }
                                } else {
                                    crate::store::log_event(
                                        "INFO",
                                        &format!(
                                            "Service '{name}' configuration location '{appdata_path}' does not exist yet."
                                        ),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        crate::store::log_event(
                            "WARN",
                            &format!(
                                "Failed to parse metadata JSON for service '{name}': {e}"
                            ),
                        );
                    }
                }
            }
            Err(e) => {
                crate::store::log_event(
                    "WARN",
                    &format!(
                        "Failed to read metadata file for service '{name}': {e}"
                    ),
                );
            }
        }
    } else {
        crate::store::log_event(
            "INFO",
            &format!(
                "No metadata file found for service '{name}'. Skipping directory permission check."
            ),
        );
    }
}

/// Sends a lifecycle action (start, stop, restart) for a specific service to the API.
pub fn send_service_action(api_port: u16, name: &str, action: &str) -> Result<(), String> {
    if !is_supervisor_running() {
        return Err("Nix process supervisor is not running.".to_string());
    }

    let is_starting = matches!(action.to_lowercase().as_str(), "start" | "restart");

    if is_starting {
        run_preflight_checks(name);
    }

    let (endpoint, method) = match action.to_lowercase().as_str() {
        "start" => (format!("process/start/{name}"), "POST"),
        "stop" => (format!("process/stop/{name}"), "PATCH"),
        "restart" => (format!("process/restart/{name}"), "POST"),
        _ => return Err(format!("Unsupported service action: {action}")),
    };

    let url = format!("http://127.0.0.1:{api_port}/{endpoint}");
    crate::store::log_event("DEBUG", &format!("Sending HTTP request to process-compose: method='{method}', url='{url}'"));
    let resp = match method {
        "POST" => ureq::post(&url).send_empty(),
        "PATCH" => ureq::patch(&url).send_empty(),
        _ => return Err(format!("Unsupported HTTP method: {method}")),
    }
    .map_err(|e| format!("HTTP request failed: {e}"))?;

    if resp.status() != 200 {
        return Err(format!("Server returned HTTP status {}", resp.status()));
    }

    crate::store::log_event("DEBUG", &format!("HTTP request completed: status='{}'", resp.status()));
    Ok(())
}
