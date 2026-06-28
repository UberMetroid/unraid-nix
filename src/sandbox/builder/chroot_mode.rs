use crate::sandbox::{parse_ports, SandboxConfig};
use super::find_nix_bash;

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
            mounts_cmd.push(cmd.clone());
        }
    }

    let chroot_dir = format!("/var/run/nix-chroot-{}", config.name);
    
    mounts_cmd.push(format!("mkdir -p {}", chroot_dir));
    mounts_cmd.push(format!("mount -t tmpfs tmpfs {}", chroot_dir));
    mounts_cmd.push(format!(
        "mkdir -p {}/nix {}/dev {}/proc {}/sys {}/etc {}/tmp {}/config",
        chroot_dir, chroot_dir, chroot_dir, chroot_dir, chroot_dir, chroot_dir, chroot_dir
    ));
    
    mounts_cmd.push(format!("mount --bind -o ro /nix {}/nix", chroot_dir));
    mounts_cmd.push(format!("mount --rbind /dev {}/dev", chroot_dir));
    mounts_cmd.push(format!("mount -t proc proc {}/proc", chroot_dir));
    mounts_cmd.push(format!("mount --rbind -o ro /sys {}/sys", chroot_dir));
    mounts_cmd.push(format!("mount -t tmpfs tmpfs {}/tmp", chroot_dir));
    
    mounts_cmd.push(format!("mkdir -p {}/etc/ssl", chroot_dir));
    mounts_cmd.push(format!(
        "touch {}/etc/resolv.conf {}/etc/passwd {}/etc/group {}/etc/hosts",
        chroot_dir, chroot_dir, chroot_dir, chroot_dir
    ));
    mounts_cmd.push(format!("mount --bind -o ro /etc/resolv.conf {}/etc/resolv.conf", chroot_dir));
    mounts_cmd.push(format!("mount --bind -o ro /etc/ssl {}/etc/ssl", chroot_dir));
    mounts_cmd.push(format!("mount --bind -o ro /etc/passwd {}/etc/passwd", chroot_dir));
    mounts_cmd.push(format!("mount --bind -o ro /etc/group {}/etc/group", chroot_dir));
    mounts_cmd.push(format!("mount --bind -o ro /etc/hosts {}/etc/hosts", chroot_dir));
    mounts_cmd.push(format!(
        "if [ -d /etc/nix ]; then mkdir -p {}/etc/nix && mount --bind -o ro /etc/nix {}/etc/nix; fi",
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

    if has_nvidia || has_render {
        if has_nvidia {
            mounts_cmd.push("/usr/local/emhttp/plugins/nix/nix-helper setup-gpus".to_string());
            mounts_cmd.push(format!("mkdir -p {}/run/opengl-driver/lib", chroot_dir));
            mounts_cmd.push(format!("mount --bind -o ro /var/run/nix-nvidia-driver/lib {}/run/opengl-driver/lib", chroot_dir));
        }
        mounts_cmd.push(format!("if [ -d /usr/lib64 ]; then mkdir -p {}/usr/lib64 && mount --bind -o ro /usr/lib64 {}/usr/lib64; fi", chroot_dir, chroot_dir));
        mounts_cmd.push(format!("if [ -d /lib64 ]; then mkdir -p {}/lib64 && mount --bind -o ro /lib64 {}/lib64; fi", chroot_dir, chroot_dir));
        mounts_cmd.push(format!("if [ -d /usr/lib ]; then mkdir -p {}/usr/lib && mount --bind -o ro /usr/lib {}/usr/lib; fi", chroot_dir, chroot_dir));
        mounts_cmd.push(format!("if [ -d /lib ]; then mkdir -p {}/lib && mount --bind -o ro /lib {}/lib; fi", chroot_dir, chroot_dir));
    }
    
    if crate::sandbox::is_uts_isolation_enabled() {
        mounts_cmd.push(format!("hostname nix-sandbox-{}", config.name));
    }
    
    let mounts_str = mounts_cmd.join(" && ").replace("\"", "\\\"");
    
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
        "exec unshare {} sh -c \"mount --make-rprivate / && {} && exec chroot --userspec={}:{} --groups={} {} {} -c \\\"{} && . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && {}\\\"\"",
        unshare_flags_str,
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
}
