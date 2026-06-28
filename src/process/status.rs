use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use super::sys;
use crate::unraid::SUPERVISOR_PORT;

const PROCESSES_PATH: &str = "/processes";
const HTTP_OK: u16 = 200;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GpuStat {
    pub sm: i32,
    pub mem: i32,
}

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
    #[serde(default)]
    pub gpu_stats: Option<std::collections::HashMap<i32, GpuStat>>,
}

impl ServiceStatus {
    pub fn uptime(&self) -> String {
        if let Some(nanos) = self.uptime_nanoseconds {
            let secs = nanos / 1_000_000_000;
            if secs < 60 {
                format!("{secs}s")
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

#[derive(Debug, Deserialize)]
pub struct ProcessComposeResponse {
    pub data: Vec<ServiceStatus>,
}

pub fn is_supervisor_running() -> bool {
    let url = format!("http://127.0.0.1:{SUPERVISOR_PORT}{PROCESSES_PATH}");
    match ureq::get(&url)
        .config()
        .timeout_per_call(Some(std::time::Duration::from_millis(150)))
        .build()
        .call()
    {
        Ok(resp) => resp.status() == HTTP_OK,
        Err(_) => {
            let pid_path = "/var/run/nix-process-compose.pid";
            if fs::metadata(pid_path).is_err() {
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
                .stdin(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    }
}

pub fn get_services_status(api_port: u16) -> Result<Vec<ServiceStatus>, String> {
    if !is_supervisor_running() {
        return Err("Nix process supervisor (process-compose) is not running.".to_string());
    }
    let url = format!("http://127.0.0.1:{api_port}{PROCESSES_PATH}");
    let mut resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("Failed to connect to process-compose API: {e}"))?;
    let wrapper: ProcessComposeResponse = resp
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse status JSON: {e}"))?;

    let mut data = wrapper.data;
    let active_gpus = sys::get_gpu_active_services();
    let pmon_stats = sys::get_nvidia_pmon_stats();
    for s in &mut data {
        s.gpu_active = Some(active_gpus.contains(&s.name));
        if let Some(pid) = s.pid {
            if let Some((rc, wc)) = sys::get_proc_io(pid) {
                s.io_read = Some(rc);
                s.io_write = Some(wc);
            }

            let descendants = sys::get_descendant_pids(pid);
            let mut service_gpu_map = std::collections::HashMap::new();
            for desc_pid in descendants {
                if let Some(gpu_list) = pmon_stats.get(&desc_pid) {
                    for &(gpu, ref stat) in gpu_list {
                        let entry = service_gpu_map.entry(gpu).or_insert(GpuStat { sm: 0, mem: 0 });
                        entry.sm = std::cmp::min(100, entry.sm + stat.sm);
                        entry.mem = std::cmp::min(100, entry.mem + stat.mem);
                    }
                }
            }
            if !service_gpu_map.is_empty() {
                s.gpu_stats = Some(service_gpu_map);
            }
        }
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

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
