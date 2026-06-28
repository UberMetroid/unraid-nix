use std::fs;
use std::process::Command;
use std::time::Duration;

use crate::util::process::run_with_timeout_status;
use super::config::{log_event, validate_store_path};

/// Binds the configured host persistent path directly to the root /nix directory.
pub fn mount_nix_store(persistent_path: &str) -> Result<(), String> {
    log_event("INFO", &format!("Attempting to mount Nix store. Persistent path: {persistent_path}"));
    if let Err(e) = validate_store_path(persistent_path) {
        log_event("ERROR", &format!("Validation failed for persistent path '{persistent_path}': {e}"));
        return Err(e);
    }

    if let Err(e) = fs::create_dir_all("/nix") {
        let err_msg = format!("Failed to create /nix: {e}");
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }
    if let Err(e) = fs::create_dir_all(persistent_path) {
        let err_msg = format!("Failed to create persistent path {persistent_path}: {e}");
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }

    let is_mounted = {
        let mut cmd = Command::new("mountpoint");
        cmd.arg("-q").arg("/nix").stdin(std::process::Stdio::null());
        run_with_timeout_status(&mut cmd, Duration::from_secs(2))
    }
    .map(|s| s.success())
    .unwrap_or(false);

    if !is_mounted {
        log_event("INFO", &format!("Mounting {persistent_path} to /nix via bind-mount..."));
        let path_meta = std::fs::symlink_metadata(persistent_path);
        match path_meta {
            Ok(m) if m.file_type().is_symlink() => {
                let err_msg = format!(
                    "Refusing to bind-mount: persistent path '{persistent_path}' is a symlink"
                );
                log_event("ERROR", &err_msg);
                return Err(err_msg);
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to stat persistent path '{persistent_path}': {e}"
                );
                log_event("ERROR", &err_msg);
                return Err(err_msg);
            }
            _ => {}
        }
        let status = {
            let mut cmd = Command::new("mount");
            cmd.arg("--bind")
                .arg(persistent_path)
                .arg("/nix")
                .stdin(std::process::Stdio::null());
            run_with_timeout_status(&mut cmd, Duration::from_secs(5))
        }
        .map_err(|e| {
            let err_msg = format!("Failed to execute mount command: {e}");
            log_event("ERROR", &err_msg);
            err_msg
        })?;

        if !status.success() {
            let err_msg = format!("Mount failed for path {persistent_path}");
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
    let is_mounted = {
        let mut cmd = Command::new("mountpoint");
        cmd.arg("-q").arg("/nix").stdin(std::process::Stdio::null());
        run_with_timeout_status(&mut cmd, Duration::from_secs(2))
    }
    .map(|s| s.success())
    .unwrap_or(false);

    if is_mounted {
        log_event("INFO", "Unmounting /nix cleanly...");
        let status = {
            let mut cmd = Command::new("umount");
            cmd.arg("-l").arg("/nix").stdin(std::process::Stdio::null());
            run_with_timeout_status(&mut cmd, Duration::from_secs(5))
        }
        .map_err(|e| {
            let err_msg = format!("Failed to execute umount command: {e}");
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