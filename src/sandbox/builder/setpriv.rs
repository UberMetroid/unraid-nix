use crate::sandbox::{parse_ports, sh_quote, SandboxConfig};

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
            mounts_cmd.push(cmd.to_owned());
        }
    }

    mounts_cmd.push("mount -t tmpfs tmpfs /boot".to_string());
    mounts_cmd.push("mount -t tmpfs tmpfs /root".to_string());
    mounts_cmd.push("if [ -d /home ]; then mount -t tmpfs tmpfs /home; fi".to_string());
    mounts_cmd.push("mkdir -p /tmp/sandbox-appdata".to_string());
    mounts_cmd.push(format!("mount --bind {} /tmp/sandbox-appdata", sh_quote(appdata_canon)));
    mounts_cmd.push(format!("mount -t tmpfs tmpfs {}", sh_quote(appdata_parent)));
    mounts_cmd.push(format!("mkdir -p {}", sh_quote(appdata_canon)));
    mounts_cmd.push(format!("mount --move /tmp/sandbox-appdata {}", sh_quote(appdata_canon)));
    mounts_cmd.push("rmdir /tmp/sandbox-appdata".to_string());

    mounts_cmd.push("mkdir -p /config".to_string());
    mounts_cmd.push(format!("mount --bind {} /config", sh_quote(appdata_canon)));

    if let Some(ref media) = config.media_path {
        if !media.trim().is_empty() {
            mounts_cmd.push("mkdir -p /media".to_string());
            mounts_cmd.push(format!("mount --bind {} /media", sh_quote(media)));
        }
    }

    for (host, sandbox) in &config.extra_binds {
        if !host.trim().is_empty() && !sandbox.trim().is_empty() {
            mounts_cmd.push(format!("mkdir -p {}", sh_quote(sandbox)));
            mounts_cmd.push(format!("mount --bind {} {}", sh_quote(host), sh_quote(sandbox)));
        }
    }

    if has_nvidia {
        mounts_cmd.push("/usr/local/emhttp/plugins/nix/nix-helper setup-gpus".to_string());
        mounts_cmd.push("mkdir -p /run/opengl-driver/lib".to_string());
        mounts_cmd.push("mount --bind /var/run/nix-nvidia-driver/lib /run/opengl-driver/lib".to_string());
    }

    let mounts_str = mounts_cmd.join(" && ");

    let mut env_vars = vec!["export HOME=/config".to_string()];
    let mut ld_paths = Vec::new();
    if has_nvidia {
        ld_paths.push("/run/opengl-driver/lib".to_string());
    }
    if has_render {
        ld_paths.push("$(nix build --no-link --print-out-paths nixpkgs#vpl-gpu-rt 2>/dev/null || true)/lib".to_string());
    }
    if !ld_paths.is_empty() {
        env_vars.push(format!("export LD_LIBRARY_PATH={}", ld_paths.join(":")));
    }
    if has_render {
        env_vars.push("export LIBVA_DRIVERS_PATH=/usr/lib64/dri:$(nix build --no-link --print-out-paths nixpkgs#intel-media-driver 2>/dev/null || true)/lib/dri".to_string());
    }
    if let Some(ref devices) = cuda_devices {
        env_vars.push(format!("export CUDA_VISIBLE_DEVICES={}", sh_quote(devices)));
    }
    if let Some(ref p_str) = config.port {
        let mappings = parse_ports(p_str);
        if let Some(first) = mappings.first() {
            env_vars.push(format!("export PORT={}", first.host));
        }
    }
    if let Some(ref addr) = config.bind_address {
        if !addr.trim().is_empty() {
            env_vars.push(format!("export BIND_ADDRESS={}", sh_quote(addr)));
            env_vars.push(format!("export HOST={}", sh_quote(addr)));
        }
    }
    let env_str = env_vars.join(" && ");

    let runuser_cmd = format!(
        "exec unshare -m sh -c \"mount --make-rprivate / && {mounts_str} && exec setpriv --reuid={puid} --regid={pgid} --init-groups sh -c \\\"{env_str} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && {inner}\\\"\"",
        mounts_str = mounts_str,
        puid = config.puid,
        pgid = config.pgid,
        env_str = env_str,
        inner = sh_quote(&config.inner_command)
    );

    Ok(runuser_cmd)
}
