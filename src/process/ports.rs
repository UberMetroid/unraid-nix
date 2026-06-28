use std::net::TcpListener;

/// Checks if a port is already bound on the host loopback or LAN interface.
///
/// Uses standard TcpListener binding. If it fails due to address in use,
/// it means there is a port conflict with a Docker container or host process.
pub fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
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
