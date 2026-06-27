/// Nix Store Management Module
///
/// This module handles system-level configurations including:
/// - Creating static UID/GID (30000+) builder accounts on boot.
/// - Bind-mounting /nix to a persistent pool path.
/// - Initializing and bootstrapping the Nix installation if missing.
/// - Starting, stopping, and checking the nix-daemon.
/// - Linking the nix.conf configuration directory.
use std::fs;
use std::os::unix::fs::symlink;
use std::process::Command;
use chrono::Local;

pub fn log_event(level: &str, msg: &str) {
    let log_path = "/var/log/nix-plugin.log";
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("{} [{}] {}\n", now, level, msg);
    
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path) {
            use std::io::Write;
            let _ = file.write_all(line.as_bytes());
        }
    
    eprintln!("[{}] {}", level, msg);
}

/// Validation check for the persistent store path.
///
/// It prevents mounting the store onto the USB/NVMe boot drive (/boot)
/// as FAT32 filesystem has no permission support and high wear risk.
pub fn validate_store_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Nix store path cannot be empty.".to_string());
    }
    if path.starts_with("/boot") {
        return Err("Nix store path cannot be located on the boot flash drive (/boot).".to_string());
    }
    Ok(())
}

/// Generates the system commands to create static nixbld groups and users.
/// Enforces high UID/GID range (30000+) to prevent clashes with Unraid GUI users.
pub fn get_user_add_commands() -> Vec<String> {
    let mut cmds = vec![
        "groupadd -g 30000 nixbld 2>/dev/null || true".to_string()
    ];
    for i in 1..=32 {
        let uid = 30000 + i;
        cmds.push(format!(
            "useradd -u {} -g nixbld -G nixbld -d /var/empty -s /bin/false -c \"Nix build user {}\" nixbld{} 2>/dev/null || true",
            uid, i, i
        ));
    }
    cmds
}

/// Creates the static nixbld builder users and groups on the host.
pub fn create_builder_accounts() -> Result<(), String> {
    log_event("INFO", "Creating static nixbld builder users and group (UID/GID 30000+)...");
    for cmd in get_user_add_commands() {
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdin(std::process::Stdio::null())
            .status()
            .map_err(|e| {
                let err_msg = format!("Failed to execute builder user/group command: {}", e);
                log_event("ERROR", &err_msg);
                err_msg
            })?;
        if !status.success() {
            // Ignore failures if group/user already exists
            continue;
        }
    }
    log_event("INFO", "Nix builder accounts verified/created.");
    Ok(())
}

/// Binds the configured host persistent path directly to the root /nix directory.
pub fn mount_nix_store(persistent_path: &str) -> Result<(), String> {
    log_event("INFO", &format!("Attempting to mount Nix store. Persistent path: {}", persistent_path));
    if let Err(e) = validate_store_path(persistent_path) {
        log_event("ERROR", &format!("Validation failed for persistent path '{}': {}", persistent_path, e));
        return Err(e);
    }

    // Create mountpoint
    if let Err(e) = fs::create_dir_all("/nix") {
        let err_msg = format!("Failed to create /nix: {}", e);
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }
    if let Err(e) = fs::create_dir_all(persistent_path) {
        let err_msg = format!("Failed to create persistent path {}: {}", persistent_path, e);
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }

    // Check if already mounted
    let is_mounted = Command::new("mountpoint")
        .arg("-q")
        .arg("/nix")
        .stdin(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !is_mounted {
        log_event("INFO", &format!("Mounting {} to /nix via bind-mount...", persistent_path));
        let status = Command::new("mount")
            .arg("--bind")
            .arg(persistent_path)
            .arg("/nix")
            .stdin(std::process::Stdio::null())
            .status()
            .map_err(|e| {
                let err_msg = format!("Failed to execute mount command: {}", e);
                log_event("ERROR", &err_msg);
                err_msg
            })?;

        if !status.success() {
            let err_msg = format!("Mount failed for path {}", persistent_path);
            log_event("ERROR", &err_msg);
            return Err(err_msg);
        }
        log_event("INFO", "Nix store bind-mount completed successfully.");
    } else {
        log_event("INFO", "Nix store is already mounted at /nix.");
    }
    Ok(())
}

/// Unmounts /nix cleanly during array stopping procedures.
pub fn unmount_nix_store() -> Result<(), String> {
    log_event("INFO", "Attempting to unmount Nix store from /nix...");
    let is_mounted = Command::new("mountpoint")
        .arg("-q")
        .arg("/nix")
        .stdin(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if is_mounted {
        log_event("INFO", "Unmounting /nix cleanly...");
        let status = Command::new("umount")
            .arg("-l")
            .arg("/nix")
            .stdin(std::process::Stdio::null())
            .status()
            .map_err(|e| {
                let err_msg = format!("Failed to execute umount command: {}", e);
                log_event("ERROR", &err_msg);
                err_msg
            })?;

        if !status.success() {
            let err_msg = "Unmount failed for /nix".to_string();
            log_event("ERROR", &err_msg);
            return Err(err_msg);
        }
        log_event("INFO", "Nix store cleanly unmounted from /nix.");
    } else {
        log_event("INFO", "/nix is not mounted. No unmount needed.");
    }
    Ok(())
}

fn read_cfg_val(key: &str, default: &str) -> String {
    let cfg_file = "/boot/config/plugins/nix/nix.cfg";
    if let Ok(content) = std::fs::read_to_string(cfg_file) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with(key) {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() >= 2 {
                    return parts[1].trim().trim_matches('"').to_string();
                }
            }
        }
    }
    default.to_string()
}

fn read_allow_source_builds() -> bool {
    read_cfg_val("ALLOW_SOURCE_BUILDS=", "no") == "yes"
}

/// Sets up persistent /etc/nix directory via symlinks.
pub fn setup_nix_conf() -> Result<(), String> {
    log_event("INFO", "Setting up persistent nix.conf...");
    let target_dir = "/nix/etc/nix";
    if let Err(e) = fs::create_dir_all(target_dir) {
        let err_msg = format!("Failed to create {}: {}", target_dir, e);
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }

    // Symlink /etc/nix to /nix/etc/nix if /etc/nix doesn't exist
    if !fs::metadata("/etc/nix").is_ok() {
        log_event("INFO", "Creating symlink /etc/nix -> /nix/etc/nix...");
        if let Err(e) = symlink(target_dir, "/etc/nix") {
            let err_msg = format!("Failed to create symlink /etc/nix -> {}: {}", target_dir, e);
            log_event("ERROR", &err_msg);
            return Err(err_msg);
        }
    }

    // Default configuration to enable flakes with safe resource concurrency limits
    let conf_path = "/nix/etc/nix/nix.conf";
    log_event("INFO", "Writing nix.conf to apply resource and builder settings...");
    
    let allow_source = read_allow_source_builds();
    
    // Parse build resource settings
    let build_cores = read_cfg_val("BUILD_CORES=", "0");
    let build_jobs = read_cfg_val("BUILD_JOBS=", "0");
    
    let (jobs_val, cores_val) = if allow_source {
        let j = if build_jobs == "0" {
            let total = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4);
            std::cmp::max(1, total / 2).to_string()
        } else {
            build_jobs
        };
        (j, build_cores)
    } else {
        ("0".to_string(), "0".to_string())
    };

    // Parse native auto-GC settings
    let gc_min_free_gb: u64 = read_cfg_val("GC_MIN_FREE=", "5").parse().unwrap_or(5);
    let gc_max_free_gb: u64 = read_cfg_val("GC_MAX_FREE=", "10").parse().unwrap_or(10);
    
    // Convert GB to bytes
    let min_free_bytes = gc_min_free_gb * 1024 * 1024 * 1024;
    let max_free_bytes = gc_max_free_gb * 1024 * 1024 * 1024;

    let default_conf = format!(
        "experimental-features = nix-command flakes\nmax-jobs = {}\ncores = {}\nmin-free = {}\nmax-free = {}\n",
        jobs_val, cores_val, min_free_bytes, max_free_bytes
    );

    if let Err(e) = fs::write(conf_path, default_conf) {
        let err_msg = format!("Failed to write nix.conf: {}", e);
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }

    // Configure system-wide flake registry overrides to pin nixpkgs channel
    let registry_path = "/nix/etc/nix/registry.json";
    let channel_ref = read_cfg_val("NIX_CHANNEL=", "nixos-unstable");
    let registry_content = format!(
        r#"{{"flakes": [{{"from": {{"id": "nixpkgs", "type": "indirect"}}, "to": {{"owner": "NixOS", "repo": "nixpkgs", "ref": "{}", "type": "github"}}}}], "version": 2}}"#,
        channel_ref
    );
    if let Err(e) = fs::write(registry_path, registry_content) {
        log_event("WARNING", &format!("Failed to write registry.json: {}", e));
    }

    log_event("INFO", "Nix configuration setup complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_store_path() {
        assert!(validate_store_path("").is_err());
        assert!(validate_store_path("/boot/nix").is_err());
        assert!(validate_store_path("/boot/config/plugins/nix").is_err());
        assert!(validate_store_path("/mnt/cache/system/nix").is_ok());
        assert!(validate_store_path("/mnt/user/appdata/nix").is_ok());
    }

    #[test]
    fn test_get_user_add_commands() {
        let cmds = get_user_add_commands();
        assert_eq!(cmds.len(), 33); // 1 groupadd + 32 useradds
        assert!(cmds[0].contains("groupadd -g 30000 nixbld"));
        assert!(cmds[1].contains("useradd -u 30001 -g nixbld"));
        assert!(cmds[32].contains("useradd -u 30032 -g nixbld"));
    }
}
