use crate::sandbox::{is_storage_sandbox_enabled, parse_ports, SandboxConfig};

pub fn build_bwrap_command(config: &SandboxConfig) -> Result<String, String> {
    if config.appdata_path.trim().is_empty() {
        return Err("Configuration Location must be specified for service execution.".to_string());
    }

    let appdata_canon = std::fs::canonicalize(&config.appdata_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| config.appdata_path.clone());

    let appdata_path_buf = std::path::PathBuf::from(&appdata_canon);
    let appdata_parent = appdata_path_buf.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/mnt/user/appdata".to_string());

    let mut mounts_cmd = Vec::new();
    
    mounts_cmd.push("mount -t tmpfs tmpfs /boot".to_string());
    mounts_cmd.push("mount -t tmpfs tmpfs /root".to_string());
    mounts_cmd.push("if [ -d /home ]; then mount -t tmpfs tmpfs /home; fi".to_string());
    
    if is_storage_sandbox_enabled() {
        let mut allowed_mnt_paths = std::collections::BTreeSet::new();
        
        if appdata_canon.starts_with("/mnt/") {
            allowed_mnt_paths.insert(appdata_canon.clone());
        }
        if let Some(ref media) = config.media_path {
            if media.starts_with("/mnt/") {
                allowed_mnt_paths.insert(media.clone());
            }
        }
        for (host, _) in &config.extra_binds {
            if host.starts_with("/mnt/") {
                allowed_mnt_paths.insert(host.clone());
            }
        }

        mounts_cmd.push("REAL_MNT_TEMP=\"/run/nix-real-mnt-$$\"".to_string());
        mounts_cmd.push("mkdir -p \"\\$REAL_MNT_TEMP\"".to_string());
        mounts_cmd.push("mount --rbind /mnt \"\\$REAL_MNT_TEMP\"".to_string());
        mounts_cmd.push("mount -t tmpfs tmpfs /mnt".to_string());

        for path in allowed_mnt_paths {
            if let Some(rel_path) = path.strip_prefix("/mnt/") {
                mounts_cmd.push(format!("mkdir -p {}", path));
                mounts_cmd.push(format!("mount --bind \"\\$REAL_MNT_TEMP/{}\" {}", rel_path, path));
            }
        }

        mounts_cmd.push("if [ -d \"\\$REAL_MNT_TEMP/disks\" ]; then mkdir -p /mnt/disks && mount --bind \"\\$REAL_MNT_TEMP/disks\" /mnt/disks; fi".to_string());
        mounts_cmd.push("if [ -d \"\\$REAL_MNT_TEMP/remotes\" ]; then mkdir -p /mnt/remotes && mount --bind \"\\$REAL_MNT_TEMP/remotes\" /mnt/remotes; fi".to_string());

        mounts_cmd.push("umount -l \"\\$REAL_MNT_TEMP\"".to_string());
        mounts_cmd.push("rmdir \"\\$REAL_MNT_TEMP\"".to_string());
    } else {
        mounts_cmd.push("mkdir -p /tmp/sandbox-appdata".to_string());
        mounts_cmd.push(format!("mount --bind {} /tmp/sandbox-appdata", appdata_canon));
        mounts_cmd.push(format!("mount -t tmpfs tmpfs {}", appdata_parent));
        mounts_cmd.push(format!("mkdir -p {}", appdata_canon));
        mounts_cmd.push(format!("mount --move /tmp/sandbox-appdata {}", appdata_canon));
        mounts_cmd.push("rmdir /tmp/sandbox-appdata".to_string());
    }

    mounts_cmd.push("mkdir -p /config".to_string());
    mounts_cmd.push(format!("mount --bind {} /config", appdata_canon));

    if let Some(ref media) = config.media_path {
        if !media.trim().is_empty() {
            mounts_cmd.push("mkdir -p /media".to_string());
            mounts_cmd.push(format!("mount --bind {} /media", media));
        }
    }

    for (host, sandbox) in &config.extra_binds {
        if !host.trim().is_empty() && !sandbox.trim().is_empty() {
            mounts_cmd.push(format!("mkdir -p {}", sandbox));
            mounts_cmd.push(format!("mount --bind {} {}", host, sandbox));
        }
    }

    let mounts_str = mounts_cmd.join(" && ").replace("\"", "\\\"");

    let mut env_vars = vec!["export HOME=/config".to_string()];
    if let Some(ref p_str) = config.port {
        let mappings = parse_ports(p_str);
        if let Some(first) = mappings.first() {
            env_vars.push(format!("export PORT={}", first.host));
        }
    }
    if let Some(ref addr) = config.bind_address {
        if !addr.trim().is_empty() {
            env_vars.push(format!("export BIND_ADDRESS={}", addr));
            env_vars.push(format!("export HOST={}", addr));
        }
    }
    let env_str = env_vars.join(" && ");

    let runuser_cmd = format!(
        "exec unshare -m sh -c \"mount --make-rprivate / && {} && exec setpriv --reuid={} --regid={} --init-groups sh -c \\\"{} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && exec {}\\\"\"",
        mounts_str,
        config.puid,
        config.pgid,
        env_str,
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
            port: Some("8080".to_string()),
            bind_address: Some("127.0.0.1".to_string()),
        };

        let cmd = build_bwrap_command(&config).unwrap();
        assert!(cmd.starts_with("exec unshare -m sh -c "));
        assert!(cmd.contains("mount -t tmpfs tmpfs /boot"));
        assert!(cmd.contains("mount --bind /mnt/cache/appdata/test-app /config"));
        assert!(cmd.contains("mount --bind /mnt/user/downloads /downloads"));
        assert!(cmd.contains("exec setpriv --reuid=99 --regid=100"));
        assert!(cmd.contains("nix run nixpkgs#hello"));
        assert!(cmd.contains("export PORT=8080"));
        assert!(cmd.contains("export BIND_ADDRESS=127.0.0.1"));
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
            port: None,
            bind_address: None,
        };

        let err = build_bwrap_command(&config);
        assert!(err.is_err());
    }

    #[test]
    fn test_build_bwrap_command_storage_sandboxed() {
        std::env::set_var("NIX_FORCE_STORAGE_SANDBOX", "1");
        
        let config = SandboxConfig {
            name: "test-app".to_string(),
            appdata_path: "/mnt/cache/appdata/test-app".to_string(),
            media_path: Some("/mnt/user/media".to_string()),
            puid: 99,
            pgid: 100,
            enable_gpu: false,
            inner_command: "nix run nixpkgs#hello".to_string(),
            extra_binds: vec![("/mnt/user/downloads".to_string(), "/downloads".to_string())],
            port: Some("8080".to_string()),
            bind_address: Some("127.0.0.1".to_string()),
        };

        let cmd = build_bwrap_command(&config).unwrap();
        std::env::remove_var("NIX_FORCE_STORAGE_SANDBOX");

        assert!(cmd.starts_with("exec unshare -m sh -c "));
        assert!(cmd.contains("mount -t tmpfs tmpfs /boot"));
        assert!(cmd.contains("REAL_MNT_TEMP="));
        assert!(cmd.contains("mount --rbind /mnt"));
        assert!(cmd.contains("mount -t tmpfs tmpfs /mnt"));
        assert!(cmd.contains("mkdir -p /mnt/cache/appdata/test-app"));
        assert!(cmd.contains("mount --bind \\\"\\$REAL_MNT_TEMP/cache/appdata/test-app\\\" /mnt/cache/appdata/test-app"));
        assert!(cmd.contains("mkdir -p /mnt/user/media"));
        assert!(cmd.contains("mount --bind \\\"\\$REAL_MNT_TEMP/user/media\\\" /mnt/user/media"));
        assert!(cmd.contains("mkdir -p /mnt/user/downloads"));
        assert!(cmd.contains("mount --bind \\\"\\$REAL_MNT_TEMP/user/downloads\\\" /mnt/user/downloads"));
        assert!(cmd.contains("umount -l \\\"\\$REAL_MNT_TEMP\\\""));
    }
}
