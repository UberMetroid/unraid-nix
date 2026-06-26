use std::net::TcpListener;

/// Checks if a port is already bound on the host loopback or LAN interface.
///
/// Uses standard TcpListener binding. If it fails due to address in use,
/// it means there is a port conflict with a Docker container or host process.
pub fn is_port_in_use(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => false,
        Err(_) => true,
    }
}
