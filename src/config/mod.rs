pub mod file;
pub mod presets;
pub mod yaml;

pub use file::{load_config, save_config};
pub use presets::{get_service_command_preset, get_preset_path};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the top-level structure of a process-compose.yml config file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProcessComposeConfig {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_configuration: Option<LogConfiguration>,
    pub processes: HashMap<String, ProcessDefinition>,
}

/// Rotation settings for process-compose logging.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_backups: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<bool>,
}

/// Logger settings for process-compose.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LogConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_timestamp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,
}

/// Defines a single process/service managed by process-compose.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProcessDefinition {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Availability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_configuration: Option<LogConfiguration>,
}

/// Restart and availability policies for the process supervisor.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Availability {
    pub restart: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_restarts: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_serialization_deserialization() {
        let mut processes = HashMap::new();
        processes.insert(
            "test-service".to_string(),
            ProcessDefinition {
                command: "nix run nixpkgs#hello".to_string(),
                availability: Some(Availability {
                    restart: "always".to_string(),
                    backoff_seconds: Some(5),
                    max_restarts: None,
                }),
                environment: None,
                log_location: None,
                log_configuration: None,
            },
        );

        let config = ProcessComposeConfig {
            version: "0.5".to_string(),
            environment: Some(vec!["NIX_REMOTE=daemon".to_string()]),
            log_configuration: Some(LogConfiguration {
                add_timestamp: Some(true),
                fields_order: Some(vec![
                    "time".to_string(),
                    "level".to_string(),
                    "message".to_string(),
                ]),
                rotation: None,
            }),
            processes,
        };

        let yaml = crate::config::yaml::serialize_config(&config);
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("0.5"));
        assert!(yaml.contains("restart: \"always\""));

        let decoded = crate::config::yaml::parse_config(&yaml).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn test_service_command_presets() {
        let cmd = get_service_command_preset("radarr", "/mnt/cache/appdata/radarr", "/mnt/user/media", 99, 100, false, None, Vec::new(), Some("7878".to_string()), Some("127.0.0.1".to_string())).unwrap();
        assert!(cmd.starts_with("exec unshare -m sh -c "));
        assert!(cmd.contains("mount -t tmpfs tmpfs /boot"));
        assert!(cmd.contains("mount --bind '/mnt/cache/appdata/radarr' /config"));
        assert!(cmd.contains("exec setpriv --reuid=99 --regid=100"));
        assert!(cmd.contains("nix run nixpkgs#radarr"));
        assert!(cmd.contains("sed -i 's|<Port>[^<]*</Port>|<Port>7878</Port>|g'"));

        let cmd_jellyfin = get_service_command_preset("jellyfin", "/mnt/cache/appdata/jellyfin", "-", 99, 100, false, None, Vec::new(), Some("8097:8096,8921:8920".to_string()), Some("10.0.0.5".to_string())).unwrap();
        assert!(cmd_jellyfin.contains("sed -i 's|<HttpsPortNumber>[^<]*</HttpsPortNumber>|<HttpsPortNumber>8921</HttpsPortNumber>|g'"));
        assert!(cmd_jellyfin.contains("sed -i 's|<LocalPortNumber>[^<]*</LocalPortNumber>|<LocalPortNumber>8097</LocalPortNumber>|g'"));
        assert!(cmd_jellyfin.contains("sed -i 's|<BindToAddress>[^<]*</BindToAddress>|<BindToAddress>'10.0.0.5'</BindToAddress>|g'"));

        let err = get_service_command_preset("invalid", "/tmp", "/tmp", 99, 100, false, None, Vec::new(), None, None);
        assert!(err.is_err());
    }
}
