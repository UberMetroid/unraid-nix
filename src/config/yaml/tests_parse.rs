//! Parser-focused tests.

use super::parse_config;

#[test]
fn parse_config_minimal() {
    let yaml = "version: '0.5'\nprocesses:\n  svc:\n    command: echo hi\n";
    let cfg = parse_config(yaml).expect("parse");
    assert_eq!(cfg.version, "0.5");
    assert_eq!(cfg.processes.len(), 1);
    let p = cfg.processes.get("svc").unwrap();
    assert_eq!(p.command, "echo hi");
    assert!(p.availability.is_none());
    assert!(p.environment.is_none());
}

#[test]
fn parse_config_with_availability() {
    let yaml = "\
version: '0.5'
processes:
  svc:
    command: run --thing
    availability:
      restart: always
      backoff_seconds: 5
      max_restarts: 3
";
    let cfg = parse_config(yaml).expect("parse");
    let p = cfg.processes.get("svc").expect("svc");
    let av = p.availability.as_ref().expect("availability");
    assert_eq!(av.restart, "always");
    assert_eq!(av.backoff_seconds, Some(5));
    assert_eq!(av.max_restarts, Some(3));
}

#[test]
fn parse_config_handles_quoted_strings() {
    let yaml = "\
version: '0.5'
processes:
  svc:
    command: \"exec sh -c 'echo \\\"hi\\\"'\"
    availability:
      restart: \"always\"
";
    let cfg = parse_config(yaml).expect("parse");
    let p = cfg.processes.get("svc").expect("svc");
    assert_eq!(p.command, "exec sh -c 'echo \"hi\"'");
    assert_eq!(p.availability.as_ref().unwrap().restart, "always");
}

#[test]
fn parse_config_handles_environment_and_log_config() {
    let yaml = "\
version: '0.5'
environment:
  - NIX_REMOTE=daemon
  - FOO=bar
log_configuration:
  add_timestamp: true
  fields_order:
    - time
    - level
    - message
  rotation:
    max_size_mb: 10
    max_backups: 3
    compress: true
processes:
  svc:
    command: run
    environment:
      - TOKEN=abc
    log_location: /var/log/svc.log
    log_configuration:
      add_timestamp: false
";
    let cfg = parse_config(yaml).expect("parse");
    assert_eq!(
        cfg.environment,
        Some(vec!["NIX_REMOTE=daemon".to_string(), "FOO=bar".to_string()])
    );
    let lc = cfg.log_configuration.as_ref().expect("top-level lc");
    assert_eq!(lc.add_timestamp, Some(true));
    assert_eq!(
        lc.fields_order,
        Some(vec![
            "time".to_string(),
            "level".to_string(),
            "message".to_string()
        ])
    );
    let rot = lc.rotation.as_ref().expect("rotation");
    assert_eq!(rot.max_size_mb, Some(10));
    assert_eq!(rot.max_backups, Some(3));
    assert_eq!(rot.compress, Some(true));

    let p = cfg.processes.get("svc").unwrap();
    assert_eq!(p.environment, Some(vec!["TOKEN=abc".to_string()]));
    assert_eq!(p.log_location, Some("/var/log/svc.log".to_string()));
    assert_eq!(
        p.log_configuration.as_ref().unwrap().add_timestamp,
        Some(false)
    );
}

#[test]
fn parse_config_rejects_bad_input() {
    assert!(parse_config("just: text").is_err()); // missing processes
    assert!(
        parse_config("processes:\n  svc:\n    availability:\n      restart: always\n").is_err()
    ); // missing command
}
