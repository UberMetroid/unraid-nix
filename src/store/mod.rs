use std::fs;
use std::os::unix::fs::symlink;
use std::process::Command;

pub mod accounts;
pub mod config;

pub use config::{generate_nix_conf_content, is_valid_service_name, log_event, validate_store_path};

/// Creates the static nixbld builder users and groups on the host.
pub fn create_builder_accounts() -> Result<(), String> {
    accounts::create_builder_accounts()
}

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

    let is_mounted = Command::new("mountpoint")
        .arg("-q")
        .arg("/nix")
        .stdin(std::process::Stdio::null())
        .status()
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
        let status = Command::new("mount")
            .arg("--bind")
            .arg(persistent_path)
            .arg("/nix")
            .stdin(std::process::Stdio::null())
            .status()
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

/// Sets up persistent /etc/nix directory via symlinks.
pub fn setup_nix_conf() -> Result<(), String> {
    log_event("INFO", "Setting up persistent nix.conf...");
    let target_dir = "/nix/etc/nix";
    if let Err(e) = fs::create_dir_all(target_dir) {
        let err_msg = format!("Failed to create {target_dir}: {e}");
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }

    if fs::metadata("/etc/nix").is_err() {
        log_event("INFO", "Creating symlink /etc/nix -> /nix/etc/nix...");
        if let Err(e) = symlink(target_dir, "/etc/nix") {
            let err_msg = format!("Failed to create symlink /etc/nix -> {target_dir}: {e}");
            log_event("ERROR", &err_msg);
            return Err(err_msg);
        }
    }

    // Bulletproof: the Determinate installer writes /etc/nix/nix.conf
    // (with `sandbox = false` by default), and per Nix docs that legacy
    // path takes precedence over /nix/etc/nix/nix.conf. We replace any
    // legacy file at /etc/nix/nix.conf with a symlink to the plugin's
    // generated config so the plugin's `sandbox = true` setting actually
    // wins. The original is backed up to .determinate.bak for forensic
    // purposes (e.g. if the admin needs to recover the installer's
    // settings).
    //
    // Skip if /etc/nix is itself already a symlink: then /etc/nix/nix.conf
    // and /nix/etc/nix/nix.conf are the SAME file via that symlink, and
    // creating another symlink on top would loop back to itself
    // (circular symlink, "Too many levels of symbolic links").
    let legacy_cfg = std::path::Path::new("/etc/nix/nix.conf");
    let persistent_cfg = std::path::Path::new("/nix/etc/nix/nix.conf");
    if legacy_cfg.is_file() && persistent_cfg.exists() {
        if std::path::Path::new("/etc/nix").is_symlink() {
            log_event(
                "WARN",
                "Skipping /etc/nix/nix.conf symlink rewrite: /etc/nix is already a symlink to /nix/etc/nix; legacy and plugin paths resolve to the same file (rewriting would create a circular symlink).",
            );
        } else {
            let backup = std::path::Path::new("/etc/nix/nix.conf.determinate.bak");
            if !backup.exists() {
                let _ = std::fs::rename(legacy_cfg, backup);
            } else {
                let _ = std::fs::remove_file(legacy_cfg);
            }
            if let Err(e) = symlink(persistent_cfg, legacy_cfg) {
                log_event("WARN", &format!("Could not symlink /etc/nix/nix.conf to plugin config: {e}"));
            } else {
                log_event("INFO", "Replaced legacy /etc/nix/nix.conf with symlink to plugin config (backup: /etc/nix/nix.conf.determinate.bak)");
            }
        }
    }

    let conf_path = "/nix/etc/nix/nix.conf";
    log_event("INFO", "Writing nix.conf to apply resource and builder settings...");

    let allow_source = config::read_allow_source_builds();
    let build_cores = config::read_cfg_val("BUILD_CORES", "0");
    let build_jobs = config::read_cfg_val("BUILD_JOBS", "0");
    let gc_min_free_gb: u64 = config::read_cfg_val("GC_MIN_FREE", "5").parse().unwrap_or(5);
    let gc_max_free_gb: u64 = config::read_cfg_val("GC_MAX_FREE", "10").parse().unwrap_or(10);

    let default_conf = generate_nix_conf_content(
        allow_source,
        &build_cores,
        &build_jobs,
        gc_min_free_gb,
        gc_max_free_gb,
    )
    .map_err(|e| {
        let err_msg = format!("Failed to generate nix.conf content: {e}");
        log_event("ERROR", &err_msg);
        err_msg
    })?;

    if let Err(e) = fs::write(conf_path, default_conf) {
        let err_msg = format!("Failed to write nix.conf: {e}");
        log_event("ERROR", &err_msg);
        return Err(err_msg);
    }

    let registry_path = "/nix/etc/nix/registry.json";
    let channel_ref = config::read_cfg_val("NIX_CHANNEL", "nixos-unstable");
    let registry_value = serde_json::json!({
        "flakes": [{
            "from": { "id": "nixpkgs", "type": "indirect" },
            "to":   { "owner": "NixOS", "repo": "nixpkgs",
                       "ref": channel_ref, "type": "github" }
        }],
        "version": 2
    });
    let registry_content = match serde_json::to_string_pretty(&registry_value) {
        Ok(s) => s,
        Err(e) => {
            let err_msg = format!("Failed to serialize registry.json: {e}");
            log_event("ERROR", &err_msg);
            return Err(err_msg);
        }
    };
    if let Err(e) = fs::write(registry_path, registry_content) {
        log_event("WARNING", &format!("Failed to write registry.json: {e}"));
    }

    log_event("INFO", "Nix configuration setup complete.");
    Ok(())
}
