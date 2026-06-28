use crate::unraid::{PROCESS_COMPOSE_CONFIG, SUPERVISOR_PORT};

pub fn restart_nix_supervisor() -> Result<(), String> {
    let _ = std::process::Command::new("sh")
        .args(["-c", &format!(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p {SUPERVISOR_PORT} down")])
        .stdin(std::process::Stdio::null())
        .output();

    let mut freed = false;
    for _ in 0..30 {
        let fuser_check = std::process::Command::new("fuser")
            .arg(format!("{SUPERVISOR_PORT}/tcp"))
            .stdin(std::process::Stdio::null())
            .output();
        if let Ok(out) = fuser_check {
            if !out.status.success() {
                freed = true;
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if !freed {
        let _ = std::process::Command::new("fuser")
            .args(["-k", &format!("{SUPERVISOR_PORT}/tcp")])
            .stdin(std::process::Stdio::null())
            .output();
    }

    let _ = std::fs::remove_file("/var/run/nix-process-compose.pid");

    if std::path::Path::new(PROCESS_COMPOSE_CONFIG).exists() {
        let _ = std::fs::create_dir_all("/var/log/nix-services");
        let cmd = format!(
            "nohup sh -c \". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && exec nix run nixpkgs#process-compose -- -p {SUPERVISOR_PORT} -f {PROCESS_COMPOSE_CONFIG} --tui=false --keep-project\" > /var/log/nix-process-compose.log 2>&1 < /dev/null & echo $! > /var/run/nix-process-compose.pid"
        );
        let status = std::process::Command::new("sh")
            .args(["-c", &cmd])
            .stdin(std::process::Stdio::null())
            .status();
        if let Ok(s) = status {
            if !s.success() {
                return Err("Failed to start process-compose supervisor.".to_string());
            }
        } else {
            return Err("Failed to start process-compose supervisor process.".to_string());
        }
    }
    Ok(())
}
