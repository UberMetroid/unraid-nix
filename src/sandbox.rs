/// Nix Host Execution Runner Module
///
/// This module constructs the execution commands using 'runuser'
/// to run processes natively on the host under the specified PUID/PGID,
/// ensuring compatibility with Unraid's rootfs architecture.
/// Configuration options for executing the sandboxed process.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub name: String,
    pub appdata_path: String,
    pub media_path: Option<String>,
    pub puid: u32,
    pub pgid: u32,
    pub enable_gpu: bool,
    pub inner_command: String,
}

/// Generates the full runuser native execution command string.
///
/// Wraps the inner command in a runuser call that drops privileges to PUID:PGID,
/// sets HOME to the appdata path (since Nix requires a writable HOME owned by the user),
/// sources the Nix daemon profile, and translates sandboxed paths (/config and /media)
/// to their host counterparts.
pub fn build_bwrap_command(config: &SandboxConfig) -> Result<String, String> {
    if config.appdata_path.trim().is_empty() {
        return Err("Appdata/Install path must be specified for service execution.".to_string());
    }

    // Translate standard sandboxed paths inside the inner command
    let mut inner_cmd = config.inner_command.clone();
    inner_cmd = inner_cmd.replace("/config", &config.appdata_path);
    if let Some(ref media) = config.media_path {
        if !media.trim().is_empty() {
            inner_cmd = inner_cmd.replace("/media", media);
        }
    }

    // Format the command to execute via runuser.
    // We source the Nix daemon profile so that nix is in the PATH and NIX_REMOTE is set correctly.
    // We set HOME to the appdata path, as Nix requires a writeable HOME directory owned by the user.
    let runuser_cmd = format!(
        "runuser -u {} -g {} -- sh -c \"export HOME={} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && {}\"",
        config.puid,
        config.pgid,
        config.appdata_path,
        inner_cmd.replace("\"", "\\\"")
    );

    Ok(runuser_cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_bwrap_command_basic() {
        let config = SandboxConfig {
            name: "test-app".to_string(),
            appdata_path: "/mnt/cache/appdata/test-app".to_string(),
            media_path: Some("/mnt/user/media".to_string()),
            puid: 99,
            pgid: 100,
            enable_gpu: false,
            inner_command: "nix run nixpkgs#hello".to_string(),
        };

        let cmd = build_bwrap_command(&config).unwrap();
        assert!(cmd.starts_with("runuser -u 99 -g 100 -- sh -c "));
        assert!(cmd.contains("export HOME=/mnt/cache/appdata/test-app"));
        assert!(cmd.contains("nix run nixpkgs#hello"));
    }

    #[test]
    fn test_build_bwrap_command_missing_appdata() {
        let config = SandboxConfig {
            name: "test-app".to_string(),
            appdata_path: "".to_string(),
            media_path: None,
            puid: 99,
            pgid: 100,
            enable_gpu: false,
            inner_command: "run".to_string(),
        };

        let err = build_bwrap_command(&config);
        assert!(err.is_err());
    }
}
