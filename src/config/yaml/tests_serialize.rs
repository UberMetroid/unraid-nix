//! Serializer-focused tests.

use super::common::{full_config_fixture, make_process};
use super::serialize::yaml_scalar;
use super::serialize_config;
use crate::config::{Availability, ProcessComposeConfig, ProcessDefinition};
use std::collections::HashMap;

#[test]
fn serialize_config_minimal() {
    let mut processes = HashMap::new();
    processes.insert("svc".to_string(), make_process("svc", "echo hi", None));
    let cfg = ProcessComposeConfig {
        version: "0.5".to_string(),
        environment: None,
        log_configuration: None,
        processes,
    };
    let yaml = serialize_config(&cfg);
    assert!(yaml.starts_with("version: \"0.5\"\n"));
    assert!(yaml.contains("processes:\n  svc:\n    command: \"echo hi\"\n"));
    assert!(!yaml.contains("availability"));
    assert!(!yaml.contains("environment"));
    assert!(!yaml.contains("log_configuration"));
}

#[test]
fn serialize_config_with_availability() {
    let mut processes = HashMap::new();
    processes.insert(
        "svc".to_string(),
        ProcessDefinition {
            command: "run".to_string(),
            availability: Some(Availability {
                restart: "always".to_string(),
                backoff_seconds: Some(5),
                max_restarts: Some(3),
            }),
            environment: None,
            log_location: None,
            log_configuration: None,
        },
    );
    let cfg = ProcessComposeConfig {
        version: "0.5".to_string(),
        environment: None,
        log_configuration: None,
        processes,
    };
    let yaml = serialize_config(&cfg);
    assert!(yaml.contains("availability:\n      restart: \"always\"\n"));
    assert!(yaml.contains("backoff_seconds: 5\n"));
    assert!(yaml.contains("max_restarts: 3\n"));
}

#[test]
fn serialize_config_multiple_processes_sorted() {
    let mut processes = HashMap::new();
    processes.insert("zeta".to_string(), make_process("zeta", "z", None));
    processes.insert("alpha".to_string(), make_process("alpha", "a", None));
    let cfg = ProcessComposeConfig {
        version: "0.5".to_string(),
        environment: None,
        log_configuration: None,
        processes,
    };
    let yaml = serialize_config(&cfg);
    let alpha_idx = yaml.find("alpha").expect("alpha present");
    let zeta_idx = yaml.find("zeta").expect("zeta present");
    assert!(alpha_idx < zeta_idx, "processes should be sorted alphabetically");
}

#[test]
fn quote_always_double() {
    // We always double-quote strings for unambiguous round-trip.
    assert_eq!(yaml_scalar("hello"), "\"hello\"");
    assert_eq!(yaml_scalar("0.5"), "\"0.5\"");
    assert_eq!(yaml_scalar("true"), "\"true\"");
    // Embedded quotes and control chars are escaped.
    assert_eq!(yaml_scalar("say \"hi\""), "\"say \\\"hi\\\"\"");
    assert_eq!(yaml_scalar("a\nb"), "\"a\\nb\"");
    assert_eq!(yaml_scalar("a\\b"), "\"a\\\\b\"");
    assert_eq!(yaml_scalar(""), "\"\"");
}

#[test]
fn roundtrip_complex() {
    let original = full_config_fixture();
    let yaml = serialize_config(&original);
    let decoded = super::parse::parse_config(&yaml).expect("re-parse");
    assert_eq!(decoded, original);

    // Serialize again and confirm byte-for-byte stability.
    let yaml2 = serialize_config(&decoded);
    assert_eq!(yaml, yaml2, "serialize -> parse -> serialize must be stable");
}