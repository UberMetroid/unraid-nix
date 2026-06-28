//! Nix Host Execution Runner Module
//!
//! This module constructs the execution commands using 'unshare' and 'setpriv'
//! to run processes in an isolated mount namespace on the host under the specified PUID/PGID,
//! preventing access to sensitive directories like /boot, /root, and other services' appdata.

pub mod builder;
pub mod cli;

pub use builder::build_bwrap_command;
pub use cli::parse_binds_string;

use crate::unraid::NIX_CFG_PATH;

/// Single-quote-escape a string for safe interpolation into a POSIX shell
/// `sh -c` command. The output is wrapped in single quotes; any single quote
/// inside is encoded as the standard `'\''` close-escape-reopen sequence.
///
/// Unlike a naive `replace("\"", "\\\"")` (which only handles the double-quote
/// case and leaves `$()`, backticks, `;`, `|`, `&`, `\\`, and newlines live),
/// this prevents the full set of shell metacharacters from being interpreted
/// when the caller later passes the escaped string to `sh -c`.
///
/// # Example
/// ```
/// use crate::sandbox::sh_quote;
/// assert_eq!(sh_quote(""), "''");
/// assert_eq!(sh_quote("hello"), "'hello'");
/// assert_eq!(sh_quote("o'clock"), "'o'\\''clock'");
/// assert_eq!(sh_quote("$HOME"), "'$HOME'");
/// ```
pub fn sh_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        match ch {
            '\'' => out.push_str("'\\''"),
            _ => out.push(ch),
        }
    }
    out.push('\'');
    out
}

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
    pub gpus: Option<String>,
    pub inner_command: String,
    pub extra_binds: Vec<(String, String)>,
    pub port: Option<String>,
    pub bind_address: Option<String>,
    pub host_init_commands: Vec<String>,
    pub enable_network_isolation: bool,
}

thread_local! {
    pub static TEST_FORCE_STORAGE_SANDBOX: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Reads a boolean value from nix.cfg for `key`. If the config file does
/// not exist, returns `default_if_missing`. If the file exists but does
/// not contain `key`, also returns `default_if_missing`. Otherwise returns
/// `true` iff the value (after trimming surrounding quotes) equals "yes".
fn read_bool_cfg(key: &str, default_if_missing: bool) -> bool {
    let Ok(content) = std::fs::read_to_string(NIX_CFG_PATH) else {
        return default_if_missing;
    };
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{key}=")) {
            let val = rest.trim_matches('"');
            return val == "yes";
        }
    }
    default_if_missing
}

pub fn is_storage_sandbox_enabled() -> bool {
    #[cfg(test)]
    {
        if let Some(val) = TEST_FORCE_STORAGE_SANDBOX.with(|v| v.get()) {
            return val;
        }
    }
    if std::env::var("NIX_FORCE_STORAGE_SANDBOX").unwrap_or_default() == "1" {
        return true;
    }
    read_bool_cfg("ENABLE_STORAGE_SANDBOX", false)
}

pub fn is_pid_isolation_enabled() -> bool {
    read_bool_cfg("ENABLE_PID_ISOLATION", true)
}

pub fn is_uts_isolation_enabled() -> bool {
    read_bool_cfg("ENABLE_UTS_ISOLATION", true)
}

pub fn is_ipc_isolation_enabled() -> bool {
    read_bool_cfg("ENABLE_IPC_ISOLATION", true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sh_quote_empty() {
        assert_eq!(sh_quote(""), "''");
    }

    #[test]
    fn test_sh_quote_plain() {
        assert_eq!(sh_quote("hello"), "'hello'");
    }

    #[test]
    fn test_sh_quote_single_quote_in_string() {
        // The classic `o'clock` case: a single quote inside the value
        // must be encoded as `'\''` (close-escape-reopen).
        assert_eq!(sh_quote("o'clock"), "'o'\\''clock'");
    }

    #[test]
    fn test_sh_quote_blocks_command_substitution() {
        // `$(curl evil)` must be neutralised — the `$` is inside single
        // quotes so bash will not evaluate the subshell.
        assert_eq!(sh_quote("$(curl evil)"), "'$(curl evil)'");
    }

    #[test]
    fn test_sh_quote_blocks_backticks() {
        // Backticks are a separate command-substitution form in POSIX sh
        // and must also be inside single quotes to be neutralised.
        assert_eq!(sh_quote("`id`"), "'`id`'");
    }

    #[test]
    fn test_sh_quote_blocks_chained_commands() {
        // `; rm -rf /` becomes a single literal token inside the quoted
        // form, so the `;` is not interpreted as a command separator.
        assert_eq!(sh_quote("; rm -rf /"), "'; rm -rf /'");
    }

    #[test]
    fn test_sh_quote_blocks_pipe() {
        assert_eq!(sh_quote("a | b"), "'a | b'");
    }

    #[test]
    fn test_sh_quote_multibyte_utf8_safe() {
        // Multi-byte chars must not panic on byte-level slicing. Iterating
        // by char keeps us on UTF-8 boundaries.
        let s = "é";
        let q = sh_quote(s);
        assert_eq!(q, "'é'");
        assert_eq!(q.chars().count(), 3);
    }
}
