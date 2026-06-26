/// Nix Host Execution Runner Module
///
/// This module constructs the execution commands using 'unshare' and 'setpriv'
/// to run processes in an isolated mount namespace on the host under the specified PUID/PGID,
/// preventing access to sensitive directories like /boot, /root, and other services' appdata.
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
    pub extra_binds: Vec<(String, String)>,
}

/// Generates the full unshare mount namespace execution command string.
///
/// Wraps the inner command in a private mount namespace (unshare -m), hides sensitive
/// host directories by mounting tmpfs over them, isolates the parent appdata path so only
/// the service's own appdata is visible, mounts /config and /media targets, sets up any
/// user-defined extra binds, and drops privileges to PUID:PGID before executing.
pub fn build_bwrap_command(config: &SandboxConfig) -> Result<String, String> {
    if config.appdata_path.trim().is_empty() {
        return Err("Appdata/Install path must be specified for service execution.".to_string());
    }

    let appdata_canon = std::fs::canonicalize(&config.appdata_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| config.appdata_path.clone());

    let appdata_path_buf = std::path::PathBuf::from(&appdata_canon);
    let appdata_parent = appdata_path_buf.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/mnt/user/appdata".to_string());

    let mut mounts_cmd = Vec::new();
    
    // 1. Hide sensitive host system directories
    mounts_cmd.push("mount -t tmpfs tmpfs /boot".to_string());
    mounts_cmd.push("mount -t tmpfs tmpfs /root".to_string());
    mounts_cmd.push("if [ -d /home ]; then mount -t tmpfs tmpfs /home; fi".to_string());
    
    // 2. Isolate appdata parent so other services' appdata folders are hidden
    mounts_cmd.push("mkdir -p /tmp/sandbox-appdata".to_string());
    mounts_cmd.push(format!("mount --bind {} /tmp/sandbox-appdata", appdata_canon));
    mounts_cmd.push(format!("mount -t tmpfs tmpfs {}", appdata_parent));
    mounts_cmd.push(format!("mkdir -p {}", appdata_canon));
    mounts_cmd.push(format!("mount --move /tmp/sandbox-appdata {}", appdata_canon));
    mounts_cmd.push("rmdir /tmp/sandbox-appdata".to_string());

    // 3. Bind `/config` to the resolved appdata directory
    mounts_cmd.push("mkdir -p /config".to_string());
    mounts_cmd.push(format!("mount --bind {} /config", appdata_canon));

    // 4. Bind `/media` if a media path is configured
    if let Some(ref media) = config.media_path {
        if !media.trim().is_empty() {
            mounts_cmd.push("mkdir -p /media".to_string());
            mounts_cmd.push(format!("mount --bind {} /media", media));
        }
    }

    // 5. Bind user-defined extra shared paths
    for (host, sandbox) in &config.extra_binds {
        if !host.trim().is_empty() && !sandbox.trim().is_empty() {
            mounts_cmd.push(format!("mkdir -p {}", sandbox));
            mounts_cmd.push(format!("mount --bind {} {}", host, sandbox));
        }
    }

    let mounts_str = mounts_cmd.join(" && ");

    // Format the command to execute via unshare and setpriv.
    // We source the Nix daemon profile so that nix is in the PATH and NIX_REMOTE is set correctly.
    // We run HOME=/config because Nix requires a writeable HOME directory owned by the user.
    let runuser_cmd = format!(
        "exec unshare -m sh -c \"mount --make-rprivate / && {} && exec setpriv --reuid={} --regid={} --init-groups sh -c \\\"export HOME=/config && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && exec {}\\\"\"",
        mounts_str,
        config.puid,
        config.pgid,
        config.inner_command.replace("\"", "\\\"")
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
            extra_binds: vec![("/mnt/user/downloads".to_string(), "/downloads".to_string())],
        };

        let cmd = build_bwrap_command(&config).unwrap();
        assert!(cmd.starts_with("exec unshare -m sh -c "));
        assert!(cmd.contains("mount -t tmpfs tmpfs /boot"));
        assert!(cmd.contains("mount --bind /mnt/cache/appdata/test-app /config"));
        assert!(cmd.contains("mount --bind /mnt/user/downloads /downloads"));
        assert!(cmd.contains("exec setpriv --reuid=99 --regid=100"));
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
            extra_binds: Vec::new(),
        };

        let err = build_bwrap_command(&config);
        assert!(err.is_err());
    }
}
