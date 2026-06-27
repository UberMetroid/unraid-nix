use crate::sandbox::{is_storage_sandbox_enabled, parse_ports, SandboxConfig};

fn find_nix_bash() -> String {
    if let Ok(entries) = std::fs::read_dir("/nix/store") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("-bash-") && !name.ends_with(".drv") {
                    let bash_bin = path.join("bin/bash");
                    if bash_bin.exists() {
                        return bash_bin.to_string_lossy().to_string();
                    }
                }
            }
        }
    }
    "/nix/var/nix/profiles/default/bin/bash".to_string()
}

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

    for cmd in &config.host_init_commands {
        if !cmd.trim().is_empty() {
            mounts_cmd.push(cmd.clone());
        }
    }

    if is_storage_sandbox_enabled() {
        let chroot_dir = format!("/var/run/nix-chroot-{}", config.name);
        
        mounts_cmd.push(format!("mkdir -p {}", chroot_dir));
        mounts_cmd.push(format!("mount -t tmpfs tmpfs {}", chroot_dir));
        mounts_cmd.push(format!(
            "mkdir -p {}/nix {}/dev {}/proc {}/sys {}/etc {}/tmp {}/config",
            chroot_dir, chroot_dir, chroot_dir, chroot_dir, chroot_dir, chroot_dir, chroot_dir
        ));
        
        mounts_cmd.push(format!("mount --bind /nix {}/nix", chroot_dir));
        mounts_cmd.push(format!("mount --rbind /dev {}/dev", chroot_dir));
        mounts_cmd.push(format!("mount --rbind /proc {}/proc", chroot_dir));
        mounts_cmd.push(format!("mount --rbind /sys {}/sys", chroot_dir));
        mounts_cmd.push(format!("mount -t tmpfs tmpfs {}/tmp", chroot_dir));
        
        mounts_cmd.push(format!("mkdir -p {}/etc/ssl", chroot_dir));
        mounts_cmd.push(format!(
            "touch {}/etc/resolv.conf {}/etc/passwd {}/etc/group {}/etc/hosts",
            chroot_dir, chroot_dir, chroot_dir, chroot_dir
        ));
        mounts_cmd.push(format!("mount --bind /etc/resolv.conf {}/etc/resolv.conf", chroot_dir));
        mounts_cmd.push(format!("mount --bind /etc/ssl {}/etc/ssl", chroot_dir));
        mounts_cmd.push(format!("mount --bind /etc/passwd {}/etc/passwd", chroot_dir));
        mounts_cmd.push(format!("mount --bind /etc/group {}/etc/group", chroot_dir));
        mounts_cmd.push(format!("mount --bind /etc/hosts {}/etc/hosts", chroot_dir));
        mounts_cmd.push(format!(
            "if [ -d /etc/nix ]; then mkdir -p {}/etc/nix && mount --bind /etc/nix {}/etc/nix; fi",
            chroot_dir, chroot_dir
        ));
        
        mounts_cmd.push(format!("mount --bind {} {}/config", appdata_canon, chroot_dir));
        
        if let Some(ref media) = config.media_path {
            if !media.trim().is_empty() {
                mounts_cmd.push(format!("mkdir -p {}/media && mount --bind {} {}/media", chroot_dir, media, chroot_dir));
            }
        }
        
        for (host, sandbox) in &config.extra_binds {
            if !host.trim().is_empty() && !sandbox.trim().is_empty() {
                mounts_cmd.push(format!(
                    "mkdir -p {}{} && mount --bind {} {}{}",
                    chroot_dir, sandbox, host, chroot_dir, sandbox
                ));
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
        
        let bash_path = find_nix_bash();
        
        let runuser_cmd = format!(
            "exec unshare -m sh -c \"mount --make-rprivate / && {} && exec chroot --userspec={}:{} --groups={} {} {} -c \\\"{} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && {}\\\"\"",
            mounts_str,
            config.puid,
            config.pgid,
            config.pgid,
            chroot_dir,
            bash_path,
            env_str,
            config.inner_command.replace("\"", "\\\"")
        );
        
        Ok(runuser_cmd)
    } else {
        mounts_cmd.push("mount -t tmpfs tmpfs /boot".to_string());
        mounts_cmd.push("mount -t tmpfs tmpfs /root".to_string());
        mounts_cmd.push("if [ -d /home ]; then mount -t tmpfs tmpfs /home; fi".to_string());
        mounts_cmd.push("mkdir -p /tmp/sandbox-appdata".to_string());
        mounts_cmd.push(format!("mount --bind {} /tmp/sandbox-appdata", appdata_canon));
        mounts_cmd.push(format!("mount -t tmpfs tmpfs {}", appdata_parent));
        mounts_cmd.push(format!("mkdir -p {}", appdata_canon));
        mounts_cmd.push(format!("mount --move /tmp/sandbox-appdata {}", appdata_canon));
        mounts_cmd.push("rmdir /tmp/sandbox-appdata".to_string());
        
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
            "exec unshare -m sh -c \"mount --make-rprivate / && {} && exec setpriv --reuid={} --regid={} --init-groups sh -c \\\"{} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && {}\\\"\"",
            mounts_str,
            config.puid,
            config.pgid,
            env_str,
            config.inner_command.replace("\"", "\\\"")
        );

        Ok(runuser_cmd)
    }
}

#[cfg(test)]
#[path = "builder_tests.rs"]
mod tests;
