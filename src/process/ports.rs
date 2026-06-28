use std::net::TcpListener;

/// Checks if a port is already bound on the host loopback or LAN interface,
/// or mapped in Docker configurations.
pub fn is_port_in_use(port: u16) -> bool {
    // 1. Check local socket bindings
    if TcpListener::bind(("127.0.0.1", port)).is_err() || TcpListener::bind(("0.0.0.0", port)).is_err() {
        return true;
    }
    
    // 2. Check Docker mappings (active or stopped)
    if crate::unraid::get_docker_mapped_ports().contains(&port) {
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
