use crate::unraid::SUPERVISOR_PORT;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
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
    let log_file_path = format!("/var/log/nix-services/{svc}.log");
    println!("\nService config written. Waiting for service to spawn logs...");

    let start_wait = Instant::now();
    let mut file = None;
    while start_wait.elapsed() < Duration::from_secs(8) {
        if let Ok(f) = File::open(&log_file_path) {
            file = Some(f);
            break;
        }
        sleep(Duration::from_millis(500));
    }

    let Some(file) = file else {
        println!("No logs spawned within 8 seconds. Service might be starting slowly in the background. Turning off autostart.");
        set_service_autostart(svc, false);
        return Ok(false);
    };

    let mut reader = BufReader::new(file);
    println!("Tailing startup logs for service: {svc}...");
    println!("--------------------------------------------------");

    let tail_start = Instant::now();
    let mut success_found = false;
    let mut last_pos = 0u64;
    let mut inode: Option<u64> = None;

    while tail_start.elapsed() < Duration::from_secs(timeout_limit_secs) {
        if let Ok(metadata) = std::fs::metadata(&log_file_path) {
            let current_inode = metadata.ino();
            let len = metadata.len();
            if inode.is_some_and(|i| i != current_inode) {
                last_pos = 0;
                inode = Some(current_inode);
                let _ = reader.get_mut().seek(SeekFrom::Start(0));
            }
            inode.get_or_insert(current_inode);

            if len > last_pos {
                if reader.get_mut().seek(SeekFrom::Start(last_pos)).is_ok() {
                    let mut line = String::new();
                    while reader.read_line(&mut line).unwrap_or(0) > 0 {
                        if let Ok(log_data) = serde_json::from_str::<LogLine>(&line) {
                            if let Some(msg) = log_data.message {
                                println!("{msg}");
                            } else {
                                print!("{line}");
                            }
                        } else {
                            print!("{line}");
                        }
                        line.clear();
                    }
                    if let Ok(pos) = reader.get_mut().stream_position() {
                        last_pos = pos;
                    }
                }
            } else if len < last_pos {
                last_pos = 0;
                let _ = reader.get_mut().seek(SeekFrom::Start(0));
            }
        }

        if let Ok(statuses) = crate::process::status::get_services_status(SUPERVISOR_PORT) {
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
                        crate::unraid::send_unraid_notification(
                            &format!("Nix: Service '{svc}' Failed"),
                            &format!("The newly installed service '{svc}' exited with status 'failed' inside the process-compose supervisor."),
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
        crate::unraid::send_unraid_notification(
            &format!("Nix: Service '{svc}' Startup Timeout"),
            &format!("The service '{svc}' took too long to enter 'running' state. Please check its log file."),
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
