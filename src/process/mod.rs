pub mod ports;
pub mod actions;

pub use actions::send_service_action;

use serde::{Deserialize, Serialize};
use std::fs;
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
    #[serde(default)]
    pub gpu_active: Option<bool>,
    #[serde(default)]
    pub io_read: Option<u64>,
    #[serde(default)]
    pub io_write: Option<u64>,
}

impl ServiceStatus {
    /// Formats the service age in seconds into a human-readable uptime string.
    #[allow(dead_code)]
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

/// Checks if the process-compose supervisor daemon is running.
pub fn is_supervisor_running() -> bool {
    let url = "http://127.0.0.1:29704/processes";
    match ureq::get(url).timeout(std::time::Duration::from_millis(150)).call() {
        Ok(resp) => resp.status() == 200,
        Err(_) => {
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

            Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    }
}

pub fn get_gpu_active_services() -> std::collections::HashSet<String> {
    let mut active_services = std::collections::HashSet::new();

    let output = Command::new("nvidia-smi")
        .args(&["--query-compute-apps=pid", "--format=csv,noheader,nounits"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let pid_str = line.trim();
                if pid_str.is_empty() || pid_str == "No running processes found" {
                    continue;
                }
                if let Ok(pid) = pid_str.parse::<i32>() {
                    let root_link = format!("/proc/{}/root", pid);
                    if let Ok(target) = std::fs::read_link(&root_link) {
                        let target_str = target.to_string_lossy();
                        if let Some(pos) = target_str.find("nix-chroot-") {
                            let start = pos + "nix-chroot-".len();
                            let service_name = &target_str[start..];
                            if !service_name.is_empty() {
                                active_services.insert(service_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    active_services
}

fn get_proc_io(pid: i32) -> Option<(u64, u64)> {
    let io_file = format!("/proc/{}/io", pid);
    if let Ok(content) = fs::read_to_string(io_file) {
        let mut rchar = None;
        let mut wchar = None;
        for line in content.lines() {
            if line.starts_with("rchar:") {
                rchar = line.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            } else if line.starts_with("wchar:") {
                wchar = line.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            }
        }
        if let (Some(rc), Some(wc)) = (rchar, wchar) {
            return Some((rc, wc));
        }
    }
    None
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

    let mut data = wrapper.data;
    let active_gpus = get_gpu_active_services();
    for s in &mut data {
        s.gpu_active = Some(active_gpus.contains(&s.name));
        if let Some(pid) = s.pid {
            if let Some((rc, wc)) = get_proc_io(pid) {
                s.io_read = Some(rc);
                s.io_write = Some(wc);
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ports::is_port_in_use;
    use std::net::TcpListener;

    #[test]
    fn test_is_port_in_use() {
        let port = 19842;
        assert_eq!(is_port_in_use(port), false);

        let _listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
        assert_eq!(is_port_in_use(port), true);

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
