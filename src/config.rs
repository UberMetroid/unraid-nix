/// Nix Flake Configuration Module
///
/// This module handles reading, modifying, and writing the declarative
/// process-compose.yml configuration file which manages background services.
/// It also defines pre-configured templates (presets) for common home services.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Represents the top-level structure of a process-compose.yml config file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProcessComposeConfig {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    pub processes: HashMap<String, ProcessDefinition>,
}

/// Defines a single process/service managed by process-compose.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProcessDefinition {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Availability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
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

/// Loads the process-compose configuration from the specified file path.
pub fn load_config(file_path: &str) -> Result<ProcessComposeConfig, String> {
    if !fs::metadata(file_path).is_ok() {
        // Return a default empty config if file doesn't exist
        return Ok(ProcessComposeConfig {
            version: "0.5".to_string(),
            environment: Some(vec!["NIX_REMOTE=daemon".to_string()]),
            processes: HashMap::new(),
        });
    }

    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: ProcessComposeConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    Ok(config)
}

/// Saves the process-compose configuration back to the specified file path.
pub fn save_config(config: &ProcessComposeConfig, file_path: &str) -> Result<(), String> {
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize config to YAML: {}", e))?;

    fs::write(file_path, yaml)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

use crate::sandbox::{build_bwrap_command, SandboxConfig};

/// Retrieves the command preset templates for common services.
/// Customizes directory paths and applies drop-privileges wrapper parameters.
pub fn get_service_command_preset(
    name: &str,
    appdata: &str,
    media: &str,
    puid: u32,
    pgid: u32,
    enable_gpu: bool,
) -> Result<String, String> {
    let media_path = if media.trim().is_empty() || media == "-" {
        None
    } else {
        Some(media.to_string())
    };

    let inner_command = match name.to_lowercase().as_str() {
        "radarr" => "HOME=/config nix run nixpkgs#radarr".to_string(),
        "sonarr" => "HOME=/config nix run nixpkgs#sonarr".to_string(),
        "jellyfin" => "nix run nixpkgs#jellyfin -- --datadir /config/data --cachedir /config/cache --configdir /config/config".to_string(),
        _ => return Err(format!("Unknown preset template: {}", name)),
    };

    build_bwrap_command(&SandboxConfig {
        name: name.to_string(),
        appdata_path: appdata.to_string(),
        media_path,
        puid,
        pgid,
        enable_gpu,
        inner_command,
    })
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
            },
        );

        let config = ProcessComposeConfig {
            version: "0.5".to_string(),
            environment: Some(vec!["NIX_REMOTE=daemon".to_string()]),
            processes,
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("0.5"));
        assert!(yaml.contains("restart: always"));

        let decoded: ProcessComposeConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn test_service_command_presets() {
        let cmd = get_service_command_preset("radarr", "/mnt/cache/appdata/radarr", "/mnt/user/media", 99, 100, false).unwrap();
        assert!(cmd.starts_with("runuser -u 99 -g 100 -- sh -c "));
        assert!(cmd.contains("export HOME=/mnt/cache/appdata/radarr"));
        assert!(cmd.contains("nix run nixpkgs#radarr"));

        let err = get_service_command_preset("invalid", "/tmp", "/tmp", 99, 100, false);
        assert!(err.is_err());
    }
}
