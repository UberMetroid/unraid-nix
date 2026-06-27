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
        host_init_commands: Vec::new(),
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
        host_init_commands: Vec::new(),
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
        host_init_commands: Vec::new(),
    };

    let cmd = build_bwrap_command(&config).unwrap();
    std::env::remove_var("NIX_FORCE_STORAGE_SANDBOX");

    assert!(cmd.starts_with("exec unshare -m sh -c "));
    assert!(cmd.contains("mount -t tmpfs tmpfs /var/run/nix-chroot-test-app"));
    assert!(cmd.contains("mount --bind /nix /var/run/nix-chroot-test-app/nix"));
    assert!(cmd.contains("mount --bind /mnt/cache/appdata/test-app /var/run/nix-chroot-test-app/config"));
    assert!(cmd.contains("mount --bind /mnt/user/media /var/run/nix-chroot-test-app/media"));
    assert!(cmd.contains("mount --bind /mnt/user/downloads /var/run/nix-chroot-test-app/downloads"));
    assert!(cmd.contains("chroot --userspec=99:100 --groups=100"));
    assert!(cmd.contains("nix run nixpkgs#hello"));
}

#[test]
fn test_build_bwrap_command_gpu() {
    let config = SandboxConfig {
        name: "test-gpu-app".to_string(),
        appdata_path: "/mnt/cache/appdata/test-gpu-app".to_string(),
        media_path: None,
        puid: 99,
        pgid: 100,
        enable_gpu: true,
        inner_command: "nix run nixpkgs#hello".to_string(),
        extra_binds: Vec::new(),
        port: None,
        bind_address: None,
        host_init_commands: Vec::new(),
    };

    let cmd = build_bwrap_command(&config).unwrap();
    assert!(cmd.contains("/bin/bash /usr/local/emhttp/plugins/nix/nix-gpu-setup.sh"));
    assert!(cmd.contains("mount --bind /var/run/nix-nvidia-driver/lib /run/opengl-driver/lib"));
    assert!(cmd.contains("export LD_LIBRARY_PATH=/run/opengl-driver/lib"));
    assert!(cmd.contains("export LIBVA_DRIVERS_PATH=/usr/lib64/dri"));
}

#[test]
fn test_build_bwrap_command_gpu_sandboxed() {
    std::env::set_var("NIX_FORCE_STORAGE_SANDBOX", "1");
    
    let config = SandboxConfig {
        name: "test-gpu-app".to_string(),
        appdata_path: "/mnt/cache/appdata/test-gpu-app".to_string(),
        media_path: None,
        puid: 99,
        pgid: 100,
        enable_gpu: true,
        inner_command: "nix run nixpkgs#hello".to_string(),
        extra_binds: Vec::new(),
        port: None,
        bind_address: None,
        host_init_commands: Vec::new(),
    };

    let cmd = build_bwrap_command(&config).unwrap();
    std::env::remove_var("NIX_FORCE_STORAGE_SANDBOX");

    assert!(cmd.contains("/bin/bash /usr/local/emhttp/plugins/nix/nix-gpu-setup.sh"));
    assert!(cmd.contains("mount --bind /var/run/nix-nvidia-driver/lib /var/run/nix-chroot-test-gpu-app/run/opengl-driver/lib"));
    assert!(cmd.contains("mount --bind -o ro /usr/lib64 /var/run/nix-chroot-test-gpu-app/usr/lib64"));
    assert!(cmd.contains("export LD_LIBRARY_PATH=/run/opengl-driver/lib"));
    assert!(cmd.contains("export LIBVA_DRIVERS_PATH=/usr/lib64/dri"));
}
