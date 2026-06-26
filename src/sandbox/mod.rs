/// Nix Host Execution Runner Module
///
/// This module constructs the execution commands using 'unshare' and 'setpriv'
/// to run processes in an isolated mount namespace on the host under the specified PUID/PGID,
/// preventing access to sensitive directories like /boot, /root, and other services' appdata.

pub mod builder;
pub mod cli;

pub use builder::build_bwrap_command;
pub use cli::{parse_sandbox_args, parse_binds_string};

#[derive(Debug, Clone, PartialEq)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
}

pub fn parse_ports(s: &str) -> Vec<PortMapping> {
    let mut mappings = Vec::new();
    if s.trim().is_empty() || s == "-" {
        return mappings;
    }
    for part in s.split(',') {
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() == 2 {
            if let (Ok(h), Ok(c)) = (subparts[0].parse::<u16>(), subparts[1].parse::<u16>()) {
                mappings.push(PortMapping { host: h, container: c });
            }
        } else if subparts.len() == 1 {
            if let Ok(p) = subparts[0].parse::<u16>() {
                mappings.push(PortMapping { host: p, container: p });
            }
        }
    }
    mappings
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub name: String,
    pub appdata_path: String,
    pub media_path: Option<String>,
    pub puid: u32,
    pub pgid: u32,
    pub enable_gpu: bool,
    pub inner_command: String,
    pub extra_binds: Vec<(String, String)>,
    pub port: Option<String>,
    pub bind_address: Option<String>,
}

pub fn is_storage_sandbox_enabled() -> bool {
    if std::env::var("NIX_FORCE_STORAGE_SANDBOX").unwrap_or_default() == "1" {
        return true;
    }
    if let Ok(content) = std::fs::read_to_string("/boot/config/plugins/nix/nix.cfg") {
        for line in content.lines() {
            if line.starts_with("ENABLE_STORAGE_SANDBOX=") {
                let val = line.trim_start_matches("ENABLE_STORAGE_SANDBOX=").trim_matches('"');
                return val == "yes";
            }
        }
    }
    false
}
