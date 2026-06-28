use std::process::{exit, Command};

pub mod helpers;
pub mod migration;

pub use crate::unraid::{parse_ini_file, detect_default_store_path};

pub fn save_settings(args: &crate::cli::args::SaveSettingsArgs) {
    let store_path = args.store_path.clone().unwrap_or_default();
    let autostart = args.autostart.clone().unwrap_or_else(|| "yes".to_string());
    let enable_sandbox = args.enable_sandbox.clone().unwrap_or_else(|| "yes".to_string());
    let show_in_nav = args.show_in_nav.clone().unwrap_or_else(|| "yes".to_string());
    let allow_source_builds = args.allow_source_builds.clone().unwrap_or_else(|| "no".to_string());
    let filter_presets_by_hardware = args.filter_presets_by_hardware.clone().unwrap_or_else(|| "yes".to_string());
    let enable_pid_isolation = args.enable_pid_isolation.clone().unwrap_or_else(|| "yes".to_string());
    let enable_uts_isolation = args.enable_uts_isolation.clone().unwrap_or_else(|| "yes".to_string());
    let enable_ipc_isolation = args.enable_ipc_isolation.clone().unwrap_or_else(|| "yes".to_string());
    let auto_gc = args.auto_gc.clone().unwrap_or_else(|| "no".to_string());
    let build_cores = args.build_cores.clone().unwrap_or_else(|| "0".to_string());
    let build_jobs = args.build_jobs.clone().unwrap_or_else(|| "0".to_string());
    let gc_min_free = args.gc_min_free.clone().unwrap_or_else(|| "5".to_string());
    let gc_max_free = args.gc_max_free.clone().unwrap_or_else(|| "10".to_string());
    let nix_channel = args.nix_channel.clone().unwrap_or_else(|| "nixos-unstable".to_string());
    let default_appdata_path = args.default_appdata_path.clone().unwrap_or_default();

    let cfg_file = "/boot/config/plugins/nix/nix.cfg";
    let old_cfg = parse_ini_file(cfg_file);
    let mut old_store_path = old_cfg.get("NIX_STORE_PATH").cloned().unwrap_or_default();
    if old_store_path.is_empty() {
        old_store_path = detect_default_store_path();
    }

    let clean_store_path = store_path.trim_end_matches('/').to_string();
    let clean_old_store_path = old_store_path.trim_end_matches('/').to_string();

    if let Err(e) = validate_settings(&clean_store_path, &default_appdata_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let mut migration_performed = false;
    if !clean_store_path.is_empty() && clean_store_path != clean_old_store_path {
        migration_performed = migration::migrate_nix_store(&clean_old_store_path, &clean_store_path);
    }

    // Write settings to ini config
    let _ = std::fs::create_dir_all("/boot/config/plugins/nix");
    let mut cfg_content = format!(
        "NIX_STORE_PATH=\"{}\"\nAUTOSTART_FLAKES=\"{}\"\nENABLE_STORAGE_SANDBOX=\"{}\"\nSHOW_IN_NAVIGATION=\"{}\"\nALLOW_SOURCE_BUILDS=\"{}\"\nFILTER_PRESETS_BY_HARDWARE=\"{}\"\nENABLE_PID_ISOLATION=\"{}\"\nENABLE_UTS_ISOLATION=\"{}\"\nENABLE_IPC_ISOLATION=\"{}\"\nAUTO_GC=\"{}\"\nBUILD_CORES=\"{}\"\nBUILD_JOBS=\"{}\"\nGC_MIN_FREE=\"{}\"\nGC_MAX_FREE=\"{}\"\nNIX_CHANNEL=\"{}\"\nSETTINGS_CONFIRMED=\"yes\"\n",
        clean_store_path, autostart, enable_sandbox, show_in_nav, allow_source_builds, filter_presets_by_hardware, enable_pid_isolation, enable_uts_isolation, enable_ipc_isolation, auto_gc, build_cores, build_jobs, gc_min_free, gc_max_free, nix_channel
    );
    if !default_appdata_path.is_empty() {
        cfg_content.push_str(&format!("DEFAULT_APPDATA_PATH=\"{}\"\n", default_appdata_path));
    }
    if std::fs::write(cfg_file, cfg_content).is_err() {
        eprintln!("Failed to write nix.cfg to flash drive.");
        exit(1);
    }

    // Configure weekly cron job based on AUTO_GC setting
    let cron_path = "/etc/cron.weekly/nix-gc";
    if auto_gc == "yes" {
        let cron_content = "#!/bin/sh\nif [ -f /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then\n    . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh\n    nix-collect-garbage -d >/var/log/nix-gc.log 2>&1\nfi\n";
        let _ = std::fs::write(cron_path, cron_content);
        let _ = Command::new("chmod").args(["+x", cron_path]).output();
    } else {
        let _ = std::fs::remove_file(cron_path);
    }

    // Update Unraid side navigation config registry
    let nix_page_file = "/usr/local/emhttp/plugins/nix/Nix.page";
    let nix_launcher_file = "/usr/local/emhttp/plugins/nix/NixLauncher.page";
    if std::path::Path::new(nix_page_file).exists() {
        if let Ok(content) = std::fs::read_to_string(nix_page_file) {
            let updated_content = if show_in_nav == "yes" {
                let launcher_content = "Menu=\"Utilities\"\nTitle=\"Nix\"\nIcon=\"nix.png\"\n---\n<script>window.location.href = '/Settings/Nix';</script>\n";
                let _ = std::fs::write(nix_launcher_file, launcher_content);
                content.lines().map(|line| {
                    if line.starts_with("Menu=") { "Menu=\"Tasks:95\"".to_string() } else { line.to_string() }
                }).collect::<Vec<String>>().join("\n")
            } else {
                let _ = std::fs::remove_file(nix_launcher_file);
                content.lines().map(|line| {
                    if line.starts_with("Menu=") { "Menu=\"Utilities\"".to_string() } else { line.to_string() }
                }).collect::<Vec<String>>().join("\n")
              };
              let _ = std::fs::write(nix_page_file, updated_content);
          }
      }

      if migration_performed {
          let _ = Command::new("/usr/local/emhttp/plugins/nix/event/disks_mounted").output();
      }
      println!("Settings saved successfully.");
  }

pub fn validate_settings(store_path: &str, default_appdata_path: &str) -> Result<(), String> {
    crate::store::validate_store_path(store_path)?;
    if !default_appdata_path.is_empty() && default_appdata_path.starts_with("/boot") {
        return Err("Default Appdata Path cannot be located on the boot flash drive (/boot).".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_settings() {
        // Valid inputs
        assert!(validate_settings("/mnt/user/system/nix", "/mnt/user/appdata").is_ok());
        
        // Invalid store path
        assert!(validate_settings("", "/mnt/user/appdata").is_err());
        assert!(validate_settings("/boot/nix", "/mnt/user/appdata").is_err());
        
        // Invalid default appdata path
        assert!(validate_settings("/mnt/user/system/nix", "/boot/appdata").is_err());
    }
}
