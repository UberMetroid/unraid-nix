use std::net::TcpListener;

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

/// Checks if a port is already bound on the host loopback or LAN interface,
/// or mapped in Docker configurations.
pub fn is_port_in_use(port: u16) -> bool {
    // 1. Check local socket bindings
    if TcpListener::bind(("127.0.0.1", port)).is_err() || TcpListener::bind(("0.0.0.0", port)).is_err() {
        return true;
    }
    
    // 2. Check Docker mappings (active or stopped)
    if get_docker_mapped_ports().contains(&port) {
        return true;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_port_in_use() {
        let port = 19842;
        assert_eq!(is_port_in_use(port), false);

        let _listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
        assert_eq!(is_port_in_use(port), true);

        drop(_listener);
    }
}
