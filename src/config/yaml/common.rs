//! Shared test fixtures.

use crate::config::{
    Availability, LogConfiguration, ProcessComposeConfig, ProcessDefinition, Rotation,
};
use std::collections::HashMap;

pub(super) fn make_process(_name: &str, command: &str, restart: Option<&str>) -> ProcessDefinition {
    ProcessDefinition {
        command: command.to_string(),
        availability: restart.map(|r| Availability {
            restart: r.to_string(),
            backoff_seconds: None,
            max_restarts: None,
        }),
        environment: None,
        log_location: None,
        log_configuration: None,
    }
}

pub(super) fn full_config_fixture() -> ProcessComposeConfig {
    let mut processes = HashMap::new();
    processes.insert(
        "radarr".to_string(),
        ProcessDefinition {
            command: "exec unshare -m sh -c 'mount -t tmpfs tmpfs /boot'".to_string(),
            availability: Some(Availability {
                restart: "always".to_string(),
                backoff_seconds: Some(5),
                max_restarts: None,
            }),
            environment: Some(vec!["FOO=bar".to_string(), "BAZ=qux".to_string()]),
            log_location: Some("/var/log/nix-services/radarr.log".to_string()),
            log_configuration: None,
        },
    );
    processes.insert(
        "jellyfin".to_string(),
        ProcessDefinition {
            command: "exec sh -c 'echo started'".to_string(),
            availability: Some(Availability {
                restart: "on-failure".to_string(),
                backoff_seconds: None,
                max_restarts: Some(7),
            }),
            environment: None,
            log_location: None,
            log_configuration: None,
        },
    );
    ProcessComposeConfig {
        version: "0.5".to_string(),
        environment: Some(vec!["NIX_REMOTE=daemon".to_string()]),
        log_configuration: Some(LogConfiguration {
            add_timestamp: Some(true),
            fields_order: Some(vec![
                "time".to_string(),
                "level".to_string(),
                "message".to_string(),
            ]),
            rotation: Some(Rotation {
                max_size_mb: Some(10),
                max_backups: Some(3),
                compress: Some(true),
            }),
        }),
        processes,
    }
}