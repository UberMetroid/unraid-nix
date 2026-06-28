use chrono::Local;

pub fn log_event(level: &str, msg: &str) {
    log_event_to_path("/var/log/nix-plugin.log", level, msg);
}

pub fn log_event_to_path(log_path: &str, level: &str, msg: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    
    // Rotate log file if it exceeds 10 MB
    if let Ok(metadata) = std::fs::metadata(log_path) {
        if metadata.len() > 10 * 1024 * 1024 {
            let backup_path = format!("{}.1", log_path);
            let _ = std::fs::rename(log_path, backup_path);
        }
    }
    
    // Sanitize carriage returns, newlines and brackets to prevent forged lines
    let safe_level = level.replace('\n', " ").replace('\r', " ").replace('[', "(").replace(']', ")");
    let safe_msg = msg.replace('\n', " ").replace('\r', " ").replace('[', "(").replace(']', ")");
    
    let line = format!("{} [{}] {}\n", now, safe_level, safe_msg);
    
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path) {
            use std::io::Write;
            let _ = file.write_all(line.as_bytes());
        }

    // Forward to native Unraid syslog safely
    if log_path == "/var/log/nix-plugin.log" {
        let _ = std::process::Command::new("logger")
            .args(["-t", "nix-plugin", &format!("[{}] {}", safe_level, safe_msg)])
            .stdin(std::process::Stdio::null())
            .output();
    }
    
    eprintln!("[{}] {}", safe_level, safe_msg);
}

/// Validation check for the persistent store path.
pub fn validate_store_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Nix store path cannot be empty.".to_string());
    }
    if path.starts_with("/boot") {
        return Err("Nix store path cannot be located on the boot flash drive (/boot).".to_string());
    }
    Ok(())
}

pub fn parse_cfg_val_from_content(content: &str, key: &str, default: &str) -> String {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                return parts[1].trim().trim_matches('"').to_string();
            }
        }
    }
    default.to_string()
}

pub fn read_cfg_val(key: &str, default: &str) -> String {
    let cfg_file = "/boot/config/plugins/nix/nix.cfg";
    if let Ok(content) = std::fs::read_to_string(cfg_file) {
        parse_cfg_val_from_content(&content, key, default)
    } else {
        default.to_string()
    }
}

pub fn read_allow_source_builds() -> bool {
    read_cfg_val("ALLOW_SOURCE_BUILDS=", "no") == "yes"
}

pub fn generate_nix_conf_content(
    allow_source: bool,
    build_cores: &str,
    build_jobs: &str,
    gc_min_free_gb: u64,
    gc_max_free_gb: u64,
) -> String {
    let (jobs_val, cores_val) = if allow_source {
        let j = if build_jobs == "0" {
            let total = std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4);
            std::cmp::max(1, total / 2).to_string()
        } else {
            build_jobs.to_string()
        };
        (j, build_cores.to_string())
    } else {
        ("0".to_string(), "0".to_string())
    };

    let min_free_bytes = gc_min_free_gb * 1024 * 1024 * 1024;
    let max_free_bytes = gc_max_free_gb * 1024 * 1024 * 1024;

    format!(
        "experimental-features = nix-command flakes\nmax-jobs = {}\ncores = {}\nmin-free = {}\nmax-free = {}\n",
        jobs_val, cores_val, min_free_bytes, max_free_bytes
    )
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
    fn test_parse_cfg_val_from_content() {
        let mock_cfg = r#"
            NIX_STORE_PATH="/mnt/cache/system/nix"
            ALLOW_SOURCE_BUILDS="yes"
            BUILD_CORES="4"
            BUILD_JOBS="2"
            GC_MIN_FREE="12"
            GC_MAX_FREE="25"
            NIX_CHANNEL="nixos-24.05"
        "#;

        assert_eq!(parse_cfg_val_from_content(mock_cfg, "ALLOW_SOURCE_BUILDS=", "no"), "yes");
        assert_eq!(parse_cfg_val_from_content(mock_cfg, "BUILD_CORES=", "0"), "4");
        assert_eq!(parse_cfg_val_from_content(mock_cfg, "BUILD_JOBS=", "0"), "2");
        assert_eq!(parse_cfg_val_from_content(mock_cfg, "GC_MIN_FREE=", "5"), "12");
        assert_eq!(parse_cfg_val_from_content(mock_cfg, "GC_MAX_FREE=", "10"), "25");
        assert_eq!(parse_cfg_val_from_content(mock_cfg, "NIX_CHANNEL=", "nixos-unstable"), "nixos-24.05");
        assert_eq!(parse_cfg_val_from_content(mock_cfg, "NON_EXISTENT_KEY=", "default_val"), "default_val");
    }

    #[test]
    fn test_generate_nix_conf_content() {
        let conf_no_source = generate_nix_conf_content(false, "4", "2", 5, 10);
        assert!(conf_no_source.contains("max-jobs = 0"));
        assert!(conf_no_source.contains("cores = 0"));
        assert!(conf_no_source.contains("min-free = 5368709120")); // 5 GB in bytes
        assert!(conf_no_source.contains("max-free = 10737418240")); // 10 GB in bytes

        let conf_source = generate_nix_conf_content(true, "8", "4", 10, 20);
        assert!(conf_source.contains("max-jobs = 4"));
        assert!(conf_source.contains("cores = 8"));
        assert!(conf_source.contains("min-free = 10737418240"));
        assert!(conf_source.contains("max-free = 21474836480"));
    }

    #[test]
    fn test_log_event_sanitization_and_rotation() {
        let log_file = std::env::temp_dir().join(format!("nix-plugin-test-{}.log", chrono::Utc::now().timestamp_micros()));
        let log_file_str = log_file.to_str().unwrap();

        // 1. Test sanitization of newlines and brackets
        log_event_to_path(log_file_str, "INFO", "hello\n[WORLD]\r");
        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("(WORLD)"));
        assert!(!content.contains("[WORLD]"));
        assert!(!content.contains("hello\n"));
        
        // 2. Test rotation
        // Write 11 MB of dummy data to the log file to trigger rotation on next write
        let dummy_data = vec![b'a'; 11 * 1024 * 1024];
        std::fs::write(&log_file, dummy_data).unwrap();
        
        // Next log write should trigger rotation
        log_event_to_path(log_file_str, "INFO", "trigger rotation");
        
        // Note: backup_file path will end with .1
        let expected_backup = format!("{}.1", log_file_str);
        assert!(std::path::Path::new(&expected_backup).exists());
        assert_eq!(std::fs::metadata(&expected_backup).unwrap().len(), 11 * 1024 * 1024);
        
        let new_content = std::fs::read_to_string(&log_file).unwrap();
        assert!(new_content.contains("trigger rotation"));
        
        // Cleanup
        let _ = std::fs::remove_file(&log_file);
        let _ = std::fs::remove_file(&expected_backup);
    }
}
