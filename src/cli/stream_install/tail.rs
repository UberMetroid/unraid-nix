use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::time::{Duration, Instant};
use std::thread::sleep;
use serde::Deserialize;

#[derive(Deserialize)]
struct LogLine {
    message: Option<String>,
}

/// Tails the log file of a newly installed service and queries process-compose.
/// Returns true if service starts successfully, false otherwise.
pub fn tail_service_logs(svc: &str, timeout_limit_secs: u64) -> Result<bool, String> {
    let log_file_path = format!("/var/log/nix-services/{}.log", svc);
    println!("\nService config written. Waiting for service to spawn logs...");
    
    // 1. Wait for log file to exist (up to 8 seconds)
    let start_wait = Instant::now();
    let mut file_opened = false;
    let mut file = None;
    while start_wait.elapsed() < Duration::from_secs(8) {
        if let Ok(f) = File::open(&log_file_path) {
            file = Some(f);
            file_opened = true;
            break;
        }
        sleep(Duration::from_millis(500));
    }

    if !file_opened || file.is_none() {
        println!("No logs spawned within 8 seconds. Service might be starting slowly in the background. Turning off autostart.");
        set_service_autostart(svc, false);
        return Ok(false);
    }

    let mut reader = BufReader::new(file.unwrap());
    println!("Tailing startup logs for service: {}...", svc);
    println!("--------------------------------------------------");

    let tail_start = Instant::now();
    let mut success_found = false;
    let mut last_pos = 0;

    while tail_start.elapsed() < Duration::from_secs(timeout_limit_secs) {
        // Read new lines if any
        if let Ok(metadata) = std::fs::metadata(&log_file_path) {
            let len = metadata.len();
            if len > last_pos {
                let _ = reader.get_mut().seek(SeekFrom::Start(last_pos));
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                    if let Ok(log_data) = serde_json::from_str::<LogLine>(&line) {
                        if let Some(msg) = log_data.message {
                            println!("{}", msg);
                        } else {
                            print!("{}", line);
                        }
                    } else {
                        print!("{}", line);
                    }
                    line.clear();
                }
                last_pos = reader.get_mut().stream_position().unwrap_or(len);
            }
        }

        // Query process-compose status
        if let Ok(statuses) = crate::process::status::get_services_status(29704) {
            for status in statuses {
                if status.name == svc {
                    let state = status.status.to_lowercase();
                    if state == "running" {
                        println!("\n[SUCCESS] Service is now running!");
                        set_service_autostart(svc, true);
                        success_found = true;
                        break;
                    }
                    if state == "failed" {
                        println!("\n[FATAL] Service failed to start!");
                        crate::store::config::send_unraid_notification(
                            &format!("Nix: Service '{}' Failed", svc),
                            &format!("The newly installed service '{}' exited with status 'failed' inside the process-compose supervisor.", svc),
                            "alert",
                        );
                        set_service_autostart(svc, false);
                        return Ok(false);
                    }
                }
            }
        }
        if success_found {
            break;
        }
        sleep(Duration::from_millis(500));
    }

    if !success_found {
        println!("\n[WARNING] Service startup verification timed out. Turning off autostart.");
        crate::store::config::send_unraid_notification(
            &format!("Nix: Service '{}' Startup Timeout", svc),
            &format!("The service '{}' took too long to enter 'running' state. Please check its log file.", svc),
            "warning",
        );
        set_service_autostart(svc, false);
        return Ok(false);
    }

    Ok(true)
}

fn set_service_autostart(svc: &str, enable: bool) {
    let toggle = if enable { "on" } else { "off" };
    crate::cli::service::autostart(svc, toggle);
}
