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
    #[serde(rename = "cpu_percent")]
    pub cpu: Option<f32>,
    #[serde(rename = "memory_bytes")]
    pub memory: Option<u64>,
    pub uptime: Option<String>,
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

/// Queries the process-compose daemon HTTP API for the status of all managed services.
pub fn get_services_status(api_port: u16) -> Result<Vec<ServiceStatus>, String> {
    if !is_supervisor_running() {
        return Err("Nix process supervisor (process-compose) is not running.".to_string());
    }

    let url = format!("http://127.0.0.1:{}/processes", api_port);
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("Failed to connect to process-compose API: {}", e))?;

    let statuses: Vec<ServiceStatus> = resp.into_json()
        .map_err(|e| format!("Failed to parse status JSON: {}", e))?;

    Ok(statuses)
}

/// Sends a lifecycle action (start, stop, restart) for a specific service to the API.
pub fn send_service_action(api_port: u16, name: &str, action: &str) -> Result<(), String> {
    if !is_supervisor_running() {
        return Err("Nix process supervisor is not running.".to_string());
    }

    // Translate user-friendly action terms to REST endpoints
    let endpoint = match action.to_lowercase().as_str() {
        "start" => format!("process/start/{}", name),
        "stop" => format!("process/stop/{}", name),
        "restart" => format!("process/restart/{}", name),
        _ => return Err(format!("Unsupported service action: {}", action)),
    };

    let url = format!("http://127.0.0.1:{}/{}", api_port, endpoint);
    let resp = ureq::post(&url)
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
        let mock_json = r#"[
            {
                "name": "radarr",
                "status": "Running",
                "pid": 1234,
                "cpu_percent": 1.2,
                "memory_bytes": 45000000,
                "uptime": "2h4m"
            }
        ]"#;

        let parsed: Vec<ServiceStatus> = serde_json::from_str(mock_json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "radarr");
        assert_eq!(parsed[0].status, "Running");
        assert_eq!(parsed[0].pid, Some(1234));
        assert_eq!(parsed[0].cpu, Some(1.2));
        assert_eq!(parsed[0].memory, Some(45000000));
        assert_eq!(parsed[0].uptime, Some("2h4m".to_string()));
    }
}
