use chrono::Local;
use std::io::Write;

pub fn log_event(level: &str, msg: &str) {
    log_event_to_path("/var/log/nix-plugin.log", level, msg, 10 * 1024 * 1024);
}

fn log_event_to_path(log_path: &str, level: &str, msg: &str, max_size: u64) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    
    // Rotate log file if it exceeds max_size (up to 3 backups)
    if let Ok(metadata) = std::fs::metadata(log_path) {
        if metadata.len() > max_size {
            let _ = std::fs::rename(format!("{}.2", log_path), format!("{}.3", log_path));
            let _ = std::fs::rename(format!("{}.1", log_path), format!("{}.2", log_path));
            let _ = std::fs::rename(log_path, format!("{}.1", log_path));
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
            let _ = file.write_all(line.as_bytes());
        }

    // Forward to native Unraid syslog safely
    if log_path == "/var/log/nix-plugin.log" {
        let _ = std::process::Command::new("logger")
            .args(["-t", "nix-plugin", &format!("[{}] {}", safe_level, safe_msg)])
            .stdin(std::process::Stdio::null())
            .output();
    }
    
    #[cfg(not(test))]
    {
        if safe_level == "ERROR" || safe_level == "WARN" || std::env::var_os("NIX_DEBUG").is_some() {
            eprintln!("[{}] {}", safe_level, safe_msg);
        }
    }
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

pub fn read_cfg_val(key: &str, default: &str) -> String {
    let cfg_file = "/boot/config/plugins/nix/nix.cfg";
    let map = crate::unraid::parse_ini_file(cfg_file);
    let clean_key = key.strip_suffix('=').unwrap_or(key);
    map.get(clean_key).cloned().unwrap_or_else(|| default.to_string())
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
            std::cmp::max(1, (total + 1) / 2).to_string()
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

pub fn is_valid_service_name(name: &str) -> bool {
    if name.is_empty() { return false; }
    if name.starts_with('.') || name.ends_with('.') || name.contains("..") { return false; }
    name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_service_name() {
        assert!(is_valid_service_name("homepage"));
        assert!(is_valid_service_name("seafile-client"));
        assert!(is_valid_service_name("my.service"));
        assert!(!is_valid_service_name(""));
        assert!(!is_valid_service_name(".service"));
        assert!(!is_valid_service_name("service."));
        assert!(!is_valid_service_name("my..service"));
        assert!(!is_valid_service_name("my/service"));
    }

    #[test]
    fn test_is_valid_service_name_equivalence() {
        let regex_equiv = |name: &str| -> bool {
            if name.is_empty() { return false; }
            let parts: Vec<&str> = name.split('.').collect();
            for part in parts {
                if part.is_empty() { return false; }
                if !part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    return false;
                }
            }
            true
        };

        // Assert 100+ ASCII strings match
        let test_cases = vec![
            "homepage", "seafile-client", "my.service", "", ".service", "service.", "my..service",
            "a", "1", "_", "-", "a-b", "a_b", "a.b.c", "a-b.c-d.e_f",
        ];
        
        let mut leakage = Vec::new();
        for c in 32u8..=126 {
            let ch = c as char;
            leakage.push(format!("a{}b", ch));
            leakage.push(format!("{}ab", ch));
            leakage.push(format!("ab{}", ch));
        }

        for case in &test_cases {
            let is_valid = is_valid_service_name(case);
            let expected = regex_equiv(case);
            assert_eq!(is_valid, expected, "Mismatch for case '{}'", case);
        }

        for case in &leakage {
            let is_valid = is_valid_service_name(case);
            let expected = regex_equiv(case);
            assert_eq!(is_valid, expected, "Mismatch for case '{}'", case);
        }
    }

    #[test]
    fn test_validate_store_path() {
        assert!(validate_store_path("").is_err());
        assert!(validate_store_path("/boot/nix").is_err());
        assert!(validate_store_path("/boot/config/plugins/nix").is_err());
        assert!(validate_store_path("/mnt/cache/system/nix").is_ok());
        assert!(validate_store_path("/mnt/user/appdata/nix").is_ok());
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
        log_event_to_path(log_file_str, "INFO", "hello\n[WORLD]\r", 1000);
        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("(WORLD)"));
        assert!(!content.contains("[WORLD]"));
        assert!(!content.contains("hello\n"));
        
        // 2. Test rotation and backup cascading
        // Write 101 bytes of dummy data to the log file to trigger rotation (threshold: 100 bytes)
        let dummy_data = vec![b'a'; 101];
        std::fs::write(&log_file, dummy_data).unwrap();
        
        // Next log write should trigger rotation: .log -> .log.1
        log_event_to_path(log_file_str, "INFO", "trigger rotation 1", 100);
        
        let expected_backup1 = format!("{}.1", log_file_str);
        assert!(std::path::Path::new(&expected_backup1).exists());
        assert_eq!(std::fs::metadata(&expected_backup1).unwrap().len(), 101);
        
        // Write 101 bytes again to trigger rotation again: .log.1 -> .log.2, .log -> .log.1
        std::fs::write(&log_file, vec![b'b'; 101]).unwrap();
        log_event_to_path(log_file_str, "INFO", "trigger rotation 2", 100);
        
        let expected_backup2 = format!("{}.2", log_file_str);
        assert!(std::path::Path::new(&expected_backup2).exists());
        assert_eq!(std::fs::metadata(&expected_backup2).unwrap().len(), 101);
        assert!(std::path::Path::new(&expected_backup1).exists());
        
        // Cleanup
        let _ = std::fs::remove_file(&log_file);
        let _ = std::fs::remove_file(&expected_backup1);
        let _ = std::fs::remove_file(&expected_backup2);
    }
}
