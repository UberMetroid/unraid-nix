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
    let conf_no_source = generate_nix_conf_content(false, "4", "2", 5, 10).unwrap();
    assert!(conf_no_source.contains("max-jobs = 0"));
    assert!(conf_no_source.contains("cores = 0"));
    assert!(conf_no_source.contains("min-free = 5368709120")); // 5 GB in bytes
    assert!(conf_no_source.contains("max-free = 10737418240")); // 10 GB in bytes

    let conf_source = generate_nix_conf_content(true, "8", "4", 10, 20).unwrap();
    assert!(conf_source.contains("max-jobs = 4"));
    assert!(conf_source.contains("cores = 8"));
    assert!(conf_source.contains("min-free = 10737418240"));
    assert!(conf_source.contains("max-free = 21474836480"));

    // Verify checked multiplication overflow detection
    assert!(generate_nix_conf_content(false, "4", "2", 18446744073709551, 10).is_err());
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
