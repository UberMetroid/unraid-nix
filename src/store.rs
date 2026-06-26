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
    for cmd in get_user_add_commands() {
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        if !status.success() {
            // Ignore failures if group/user already exists
            continue;
        }
    }
    Ok(())
}

/// Binds the configured host persistent path directly to the root /nix directory.
pub fn mount_nix_store(persistent_path: &str) -> Result<(), String> {
    validate_store_path(persistent_path)?;

    // Create mountpoint
    fs::create_dir_all("/nix").map_err(|e| format!("Failed to create /nix: {}", e))?;
    fs::create_dir_all(persistent_path).map_err(|e| format!("Failed to create persistent path: {}", e))?;

    // Check if already mounted
    let is_mounted = Command::new("mountpoint")
        .arg("-q")
        .arg("/nix")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !is_mounted {
        let status = Command::new("mount")
            .arg("--bind")
            .arg(persistent_path)
            .arg("/nix")
            .status()
            .map_err(|e| format!("Failed to execute mount command: {}", e))?;

        if !status.success() {
            return Err(format!("Mount failed for path {}", persistent_path));
        }
    }
    Ok(())
}

/// Unmounts /nix cleanly during array stopping procedures.
pub fn unmount_nix_store() -> Result<(), String> {
    let is_mounted = Command::new("mountpoint")
        .arg("-q")
        .arg("/nix")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if is_mounted {
        let status = Command::new("umount")
            .arg("-l")
            .arg("/nix")
            .status()
            .map_err(|e| format!("Failed to execute umount command: {}", e))?;

        if !status.success() {
            return Err("Unmount failed for /nix".to_string());
        }
    }
    Ok(())
}

/// Sets up persistent /etc/nix directory via symlinks.
pub fn setup_nix_conf() -> Result<(), String> {
    let target_dir = "/nix/etc/nix";
    fs::create_dir_all(target_dir).map_err(|e| format!("Failed to create {}: {}", target_dir, e))?;

    // Symlink /etc/nix to /nix/etc/nix if /etc/nix doesn't exist
    if !fs::metadata("/etc/nix").is_ok() {
        symlink(target_dir, "/etc/nix")
            .map_err(|e| format!("Failed to create symlink /etc/nix -> {}: {}", target_dir, e))?;
    }

    // Default configuration to enable flakes
    let conf_path = "/nix/etc/nix/nix.conf";
    if !fs::metadata(conf_path).is_ok() {
        let default_conf = "experimental-features = nix-command flakes\nmax-jobs = auto\n";
        fs::write(conf_path, default_conf)
            .map_err(|e| format!("Failed to write default nix.conf: {}", e))?;
    }
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
