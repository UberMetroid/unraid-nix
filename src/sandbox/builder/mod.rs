pub mod chroot_mode;
pub mod setpriv_mode;

use crate::sandbox::{is_storage_sandbox_enabled, SandboxConfig};

pub use chroot_mode::build_chroot_command;
pub use setpriv_mode::build_setpriv_command;

pub fn find_nix_bash() -> String {
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

    let has_nvidia = if let Some(ref g) = config.gpus {
        g.contains("nvidia")
    } else {
        config.enable_gpu
    };

    let has_render = if let Some(ref g) = config.gpus {
        g.contains("renderD")
    } else {
        config.enable_gpu
    };

    let mut nvidia_indexes = Vec::new();
    if let Some(ref g) = config.gpus {
        for part in g.split(',') {
            if part.starts_with("nvidia-") {
                if let Some(idx_str) = part.strip_prefix("nvidia-") {
                    if let Ok(idx) = idx_str.parse::<u32>() {
                        nvidia_indexes.push(idx.to_string());
                    }
                }
            }
        }
    }
    
    let cuda_devices = if nvidia_indexes.is_empty() {
        if let Some(ref g) = config.gpus {
            if g.trim().is_empty() { Some("".to_string()) } else { None }
        } else if config.enable_gpu {
            None
        } else {
            Some("".to_string())
        }
    } else {
        Some(nvidia_indexes.join(","))
    };

    let appdata_canon = std::fs::canonicalize(&config.appdata_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| config.appdata_path.clone());

    if is_storage_sandbox_enabled() {
        build_chroot_command(config, &appdata_canon, has_nvidia, has_render, &cuda_devices)
    } else {
        let appdata_path_buf = std::path::PathBuf::from(&appdata_canon);
        let appdata_parent = appdata_path_buf.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/mnt/user/appdata".to_string());
        build_setpriv_command(config, &appdata_canon, &appdata_parent, has_nvidia, has_render, &cuda_devices)
    }
}

#[cfg(test)]
#[path = "../builder_tests.rs"]
mod tests;
