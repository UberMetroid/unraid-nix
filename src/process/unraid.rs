/// Native Unraid WebUI notification helper
pub fn send_unraid_notification(subject: &str, description: &str, importance: &str) {
    let importance_flag = match importance {
        "alert" => "alert",
        "warning" => "warning",
        _ => "normal",
    };
    let _ = std::process::Command::new("/usr/local/emhttp/webGui/scripts/notify")
        .args([
            "-e", "Nix Plugin",
            "-s", subject,
            "-d", description,
            "-i", importance_flag,
        ])
        .stdin(std::process::Stdio::null())
        .output();
}

/// Query active or stopped Docker container ports mapping
pub fn get_docker_mapped_ports() -> Vec<u16> {
    let mut ports = Vec::new();
    let output = std::process::Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Ports}}"])
        .stdin(std::process::Stdio::null())
        .output();
        
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            // e.g. "0.0.0.0:8080->80/tcp, :::8080->80/tcp"
            for part in line.split(',') {
                if let Some(arrow_idx) = part.find("->") {
                    let before_arrow = &part[..arrow_idx];
                    if let Some(colon_idx) = before_arrow.rfind(':') {
                        let port_str = &before_arrow[colon_idx + 1..];
                        if let Ok(port) = port_str.parse::<u16>() {
                            ports.push(port);
                        }
                    }
                }
            }
        }
    }
    ports
}
