use super::find_nix_bash;
use crate::sandbox::{parse_ports, sh_quote, SandboxConfig};

pub fn build_chroot_command(
    config: &SandboxConfig,
    appdata_canon: &str,
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

    let chroot_dir = format!("/var/run/nix-chroot-{}", sh_quote(&config.name));

    mounts_cmd.push(format!("mkdir -p {chroot_dir}"));
    mounts_cmd.push(format!("mount -t tmpfs tmpfs {chroot_dir}"));
    mounts_cmd.push(format!(
        "mkdir -p {chroot_dir}/nix {chroot_dir}/dev {chroot_dir}/proc {chroot_dir}/sys {chroot_dir}/etc {chroot_dir}/tmp {chroot_dir}/config"
    ));

    mounts_cmd.push(format!("mount --bind -o ro /nix {chroot_dir}/nix"));
    mounts_cmd.push(format!("mount --rbind /dev {chroot_dir}/dev"));
    mounts_cmd.push(format!("mount -t proc proc {chroot_dir}/proc"));
    mounts_cmd.push(format!("mount --rbind -o ro /sys {chroot_dir}/sys"));
    mounts_cmd.push(format!("mount -t tmpfs tmpfs {chroot_dir}/tmp"));

    mounts_cmd.push(format!("mkdir -p {chroot_dir}/etc/ssl"));
    mounts_cmd.push(format!(
        "touch {chroot_dir}/etc/resolv.conf {chroot_dir}/etc/passwd {chroot_dir}/etc/group {chroot_dir}/etc/hosts"
    ));
    mounts_cmd.push(format!(
        "mount --bind -o ro /etc/resolv.conf {chroot_dir}/etc/resolv.conf"
    ));
    mounts_cmd.push(format!("mount --bind -o ro /etc/ssl {chroot_dir}/etc/ssl"));
    mounts_cmd.push(format!(
        "mount --bind -o ro /etc/passwd {chroot_dir}/etc/passwd"
    ));
    mounts_cmd.push(format!(
        "mount --bind -o ro /etc/group {chroot_dir}/etc/group"
    ));
    mounts_cmd.push(format!(
        "mount --bind -o ro /etc/hosts {chroot_dir}/etc/hosts"
    ));
    mounts_cmd.push(format!(
        "if [ -d /etc/nix ]; then mkdir -p {chroot_dir}/etc/nix && mount --bind -o ro /etc/nix {chroot_dir}/etc/nix; fi"
    ));

    mounts_cmd.push(format!(
        "mount --bind {} {chroot_dir}/config",
        sh_quote(appdata_canon)
    ));

    if let Some(ref media) = config.media_path {
        if !media.trim().is_empty() {
            mounts_cmd.push(format!(
                "mkdir -p {chroot_dir}/media && mount --bind {} {chroot_dir}/media",
                sh_quote(media)
            ));
        }
    }

    for (host, sandbox) in &config.extra_binds {
        if !host.trim().is_empty() && !sandbox.trim().is_empty() {
            mounts_cmd.push(format!(
                "mkdir -p {chroot_dir}{} && mount --bind {} {chroot_dir}{}",
                sh_quote(sandbox),
                sh_quote(host),
                sh_quote(sandbox)
            ));
        }
    }

    if has_nvidia || has_render {
        if has_nvidia {
            mounts_cmd.push("/usr/local/emhttp/plugins/nix/nix-helper setup-gpus".to_string());
            mounts_cmd.push(format!("mkdir -p {chroot_dir}/run/opengl-driver/lib"));
            mounts_cmd.push(format!("mount --bind -o ro /var/run/nix-nvidia-driver/lib {chroot_dir}/run/opengl-driver/lib"));
        }
        mounts_cmd.push(format!("if [ -d /usr/lib64 ]; then mkdir -p {chroot_dir}/usr/lib64 && mount --bind -o ro /usr/lib64 {chroot_dir}/usr/lib64; fi"));
        mounts_cmd.push(format!("if [ -d /lib64 ]; then mkdir -p {chroot_dir}/lib64 && mount --bind -o ro /lib64 {chroot_dir}/lib64; fi"));
        mounts_cmd.push(format!("if [ -d /usr/lib ]; then mkdir -p {chroot_dir}/usr/lib && mount --bind -o ro /usr/lib {chroot_dir}/usr/lib; fi"));
        mounts_cmd.push(format!("if [ -d /lib ]; then mkdir -p {chroot_dir}/lib && mount --bind -o ro /lib {chroot_dir}/lib; fi"));
    }

    if crate::sandbox::is_uts_isolation_enabled() {
        mounts_cmd.push(format!("hostname nix-sandbox-{}", sh_quote(&config.name)));
    }

    let mounts_str = mounts_cmd.join(" && ");

    let mut env_vars = vec!["export HOME=/config".to_string()];
    let mut ld_paths = Vec::new();
    if has_nvidia {
        ld_paths.push("/run/opengl-driver/lib".to_string());
    }
    if has_render {
        ld_paths.push(
            "$(nix build --no-link --print-out-paths nixpkgs#vpl-gpu-rt 2>/dev/null || true)/lib"
                .to_string(),
        );
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

    let bash_path = find_nix_bash();

    let mut unshare_flags = vec!["-m".to_string()];
    if crate::sandbox::is_pid_isolation_enabled() {
        unshare_flags.push("-p".to_string());
        unshare_flags.push("--fork".to_string());
    }
    if crate::sandbox::is_uts_isolation_enabled() {
        unshare_flags.push("-u".to_string());
    }
    if crate::sandbox::is_ipc_isolation_enabled() {
        unshare_flags.push("-i".to_string());
    }
    if config.enable_network_isolation {
        unshare_flags.push("-n".to_string());
    }
    let unshare_flags_str = unshare_flags.join(" ");

    let runuser_cmd = format!(
        "exec unshare {unshare_flags_str} sh -c \"mount --make-rprivate / && {mounts_str} && exec chroot --userspec={puid}:{pgid} --groups={pgid} {chroot_dir} {bash_path} -c \\\"{env_str} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && {inner}\\\"\"",
        unshare_flags_str = unshare_flags_str,
        mounts_str = mounts_str,
        puid = config.puid,
        pgid = config.pgid,
        chroot_dir = chroot_dir,
        bash_path = bash_path,
        env_str = env_str,
        inner = sh_quote(&config.inner_command)
    );

    Ok(runuser_cmd)
}
