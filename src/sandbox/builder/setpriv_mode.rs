use crate::sandbox::{parse_ports, SandboxConfig};

pub fn build_setpriv_command(
    config: &SandboxConfig,
    appdata_canon: &str,
    appdata_parent: &str,
    has_nvidia: bool,
    has_render: bool,
    cuda_devices: &Option<String>,
) -> Result<String, String> {
    let mut mounts_cmd = Vec::new();

    for cmd in &config.host_init_commands {
        if !cmd.trim().is_empty() {
            mounts_cmd.push(cmd.clone());
        }
    }

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

    if has_nvidia {
        mounts_cmd.push("/bin/bash /usr/local/emhttp/plugins/nix/nix-gpu-setup.sh".to_string());
        mounts_cmd.push("mkdir -p /run/opengl-driver/lib".to_string());
        mounts_cmd.push("mount --bind /var/run/nix-nvidia-driver/lib /run/opengl-driver/lib".to_string());
    }

    let mounts_str = mounts_cmd.join(" && ").replace("\"", "\\\"");

    let mut env_vars = vec!["export HOME=/config".to_string()];
    if has_nvidia {
        env_vars.push("export LD_LIBRARY_PATH=/run/opengl-driver/lib".to_string());
    }
    if has_render {
        env_vars.push("export LIBVA_DRIVERS_PATH=/usr/lib64/dri:$(nix build --no-link --print-out-paths nixpkgs#intel-media-driver 2>/dev/null || true)/lib/dri".to_string());
    }
    if let Some(ref devices) = cuda_devices {
        env_vars.push(format!("export CUDA_VISIBLE_DEVICES={}", devices));
    }
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
