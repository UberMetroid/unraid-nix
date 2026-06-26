/// Nix Process Supervisor Integration Module
///
/// This module interfaces with the running process-compose REST API
/// to query service statuses, execute actions (start, stop, restart),
/// and perform pre-flight checks like host port-conflict detection.
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::TcpListener;
use std::process::Command;

/// Service status details retrieved from the process-compose API.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub pid: Option<i32>,
    pub cpu: Option<f32>,
    #[serde(rename = "mem")]
    pub memory: Option<u64>,
    #[serde(rename = "age")]
    pub uptime_nanoseconds: Option<u64>,
    pub exit_code: Option<i32>,
}

impl ServiceStatus {
    /// Formats the service age in seconds into a human-readable uptime string.
    pub fn uptime(&self) -> String {
        if let Some(nanos) = self.uptime_nanoseconds {
            let secs = nanos / 1_000_000_000;
            if secs < 60 {
                format!("{}s", secs)
            } else if secs < 3600 {
                format!("{}m", secs / 60)
            } else {
                format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
            }
        } else {
            "-".to_string()
        }
    }
}

/// Wrapper for the process-compose JSON status response.
#[derive(Debug, Deserialize)]
pub struct ProcessComposeResponse {
    pub data: Vec<ServiceStatus>,
}

/// Checks if a port is already bound on the host loopback or LAN interface.
///
/// Uses standard TcpListener binding. If it fails due to address in use,
/// it means there is a port conflict with a Docker container or host process.
pub fn is_port_in_use(port: u16) -> bool {
    // Try binding to localhost
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => false, // Successfully bound, so port was free
        Err(_) => true,  // Failed to bind, port is in use
    }
}

/// Checks if the process-compose supervisor daemon is running.
///
/// Reads the pidfile /var/run/nix-process-compose.pid and verifies the process is alive.
pub fn is_supervisor_running() -> bool {
    // Attempt a lightweight HTTP connection check on localhost port 29704
    let url = "http://127.0.0.1:29704/processes";
    match ureq::get(url).timeout(std::time::Duration::from_millis(150)).call() {
        Ok(resp) => resp.status() == 200,
        Err(_) => {
            // Fallback: Check PID file in case API is temporarily unresponsive
            let pid_path = "/var/run/nix-process-compose.pid";
            if !fs::metadata(pid_path).is_ok() {
                return false;
            }

            let pid_str = match fs::read_to_string(pid_path) {
                Ok(s) => s.trim().to_string(),
                Err(_) => return false,
            };

            let pid = match pid_str.parse::<i32>() {
                Ok(p) => p,
                Err(_) => return false,
            };

            // Run kill -0 <pid> to verify the process is alive
            Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    }
}

/// Queries the process-compose daemon HTTP API for the status of all managed services.
pub fn get_services_status(api_port: u16) -> Result<Vec<ServiceStatus>, String> {
    if !is_supervisor_running() {
        return Err("Nix process supervisor (process-compose) is not running.".to_string());
    }

    let url = format!("http://127.0.0.1:{}/processes", api_port);
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("Failed to connect to process-compose API: {}", e))?;

    let wrapper: ProcessComposeResponse = resp.into_json()
        .map_err(|e| format!("Failed to parse status JSON: {}", e))?;

    Ok(wrapper.data)
}

fn run_preflight_checks(name: &str) {
    crate::store::log_event("INFO", &format!("Running pre-flight checks for service '{}'...", name));

    // 1. Port conflict warning for known service presets
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
                    "Port conflict warning: Port {} (default for preset '{}') is already bound on the host.",
                    port, name
                ),
            );
        } else {
            crate::store::log_event(
                "INFO",
                &format!(
                    "Port check passed: Port {} (default for preset '{}') is free.",
                    port, name
                ),
            );
        }
    }

    // 2. Data directory permissions check
    let metadata_path = format!("/boot/config/plugins/nix/metadata/{}.json", name);
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
                                if path.exists() {
                                    use std::os::unix::fs::MetadataExt;
                                    match std::fs::metadata(path) {
                                        Ok(fs_meta) => {
                                            let owner_uid = fs_meta.uid();
                                            let expected_puid = if let Some(ref val) = meta.puid {
                                                if let Some(num) = val.as_u64() {
                                                    num as u32
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
                                                        "Directory permissions warning: Service '{}' configuration location '{}' owner UID ({}) does not match configured PUID ({}).",
                                                        name, appdata_path, owner_uid, expected_puid
                                                    ),
                                                );
                                            } else {
                                                crate::store::log_event(
                                                    "INFO",
                                                    &format!(
                                                        "Directory permissions check passed: Service '{}' configuration location '{}' is owned by UID {}.",
                                                        name, appdata_path, owner_uid
                                                    ),
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            crate::store::log_event(
                                                "WARN",
                                                &format!(
                                                    "Failed to read file metadata for configuration location '{}': {}",
                                                    appdata_path, e
                                                ),
                                            );
                                        }
                                    }
                                } else {
                                    crate::store::log_event(
                                        "INFO",
                                        &format!(
                                            "Service '{}' configuration location '{}' does not exist yet.",
                                            name, appdata_path
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
                                "Failed to parse metadata JSON for service '{}': {}",
                                name, e
                            ),
                        );
                    }
                }
            }
            Err(e) => {
                crate::store::log_event(
                    "WARN",
                    &format!(
                        "Failed to read metadata file for service '{}': {}",
                        name, e
                    ),
                );
            }
        }
    } else {
        crate::store::log_event(
            "INFO",
            &format!(
                "No metadata file found for service '{}'. Skipping directory permission check.",
                name
            ),
        );
    }
}

/// Sends a lifecycle action (start, stop, restart) for a specific service to the API.
pub fn send_service_action(api_port: u16, name: &str, action: &str) -> Result<(), String> {
    if !is_supervisor_running() {
        return Err("Nix process supervisor is not running.".to_string());
    }

    let is_starting = match action.to_lowercase().as_str() {
        "start" | "restart" => true,
        _ => false,
    };

    if is_starting {
        run_preflight_checks(name);
    }

    // Translate user-friendly action terms to REST endpoints
    let (endpoint, method) = match action.to_lowercase().as_str() {
        "start" => (format!("process/start/{}", name), "POST"),
        "stop" => (format!("process/stop/{}", name), "PATCH"),
        "restart" => (format!("process/restart/{}", name), "POST"),
        _ => return Err(format!("Unsupported service action: {}", action)),
    };

    let url = format!("http://127.0.0.1:{}/{}", api_port, endpoint);
    let resp = ureq::request(method, &url)
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if resp.status() != 200 {
        return Err(format!("Server returned HTTP status {}", resp.status()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_port_in_use() {
        // Choose a random port that is likely free
        let port = 19842;
        assert_eq!(is_port_in_use(port), false);

        // Bind a listener to it, making it in use
        let _listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
        assert_eq!(is_port_in_use(port), true);

        // Listener dropped here, port should be free again
        drop(_listener);
    }

    #[test]
    fn test_parse_mock_service_status() {
        let mock_json = r#"{
            "data": [
                {
                    "name": "radarr",
                    "status": "Running",
                    "pid": 1234,
                    "cpu": 1.2,
                    "mem": 45000000,
                    "age": 7440000000000
                }
            ]
        }"#;

        let parsed: ProcessComposeResponse = serde_json::from_str(mock_json).unwrap();
        assert_eq!(parsed.data.len(), 1);
        assert_eq!(parsed.data[0].name, "radarr");
        assert_eq!(parsed.data[0].status, "Running");
        assert_eq!(parsed.data[0].pid, Some(1234));
        assert_eq!(parsed.data[0].cpu, Some(1.2));
        assert_eq!(parsed.data[0].memory, Some(45000000));
        assert_eq!(parsed.data[0].uptime(), "2h4m");
    }
}
