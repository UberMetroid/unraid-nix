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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_configuration: Option<LogConfiguration>,
    pub processes: HashMap<String, ProcessDefinition>,
}

/// Logger settings for process-compose.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LogConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_timestamp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_order: Option<Vec<String>>,
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

/// Loads the process-compose configuration from the specified file path.
pub fn load_config(file_path: &str) -> Result<ProcessComposeConfig, String> {
    if !fs::metadata(file_path).is_ok() {
        // Return a default empty config if file doesn't exist
        return Ok(ProcessComposeConfig {
            version: "0.5".to_string(),
            environment: Some(vec!["NIX_REMOTE=daemon".to_string()]),
            log_configuration: Some(LogConfiguration {
                add_timestamp: Some(true),
                fields_order: Some(vec![
                    "time".to_string(),
                    "level".to_string(),
                    "message".to_string(),
                ]),
            }),
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

use crate::sandbox::{build_bwrap_command, parse_ports, SandboxConfig};

/// Retrieves the command preset templates for common services.
/// Customizes directory paths and applies drop-privileges wrapper parameters.
pub fn get_service_command_preset(
    name: &str,
    appdata: &str,
    media: &str,
    puid: u32,
    pgid: u32,
    enable_gpu: bool,
    extra_binds: Vec<(String, String)>,
    port: Option<String>,
    bind_address: Option<String>,
) -> Result<String, String> {
    let media_path = if media.trim().is_empty() || media == "-" {
        None
    } else {
        Some(media.to_string())
    };

    let inner_command = match name.to_lowercase().as_str() {
        "radarr" | "sonarr" => {
            let default_port = if name.to_lowercase() == "radarr" { 7878 } else { 8989 };
            let mappings = port.as_ref().map(|s| parse_ports(s)).unwrap_or_default();
            let p = mappings.first().map(|m| m.host).unwrap_or(default_port);
            let addr = bind_address.clone().unwrap_or_else(|| "*".to_string());
            // Create default config.xml if it doesn't exist, and patch Port/BindAddress using sed inside the namespace
            format!(
                "mkdir -p /config && [ ! -f /config/config.xml ] && echo '<Config><Port>{}</Port><BindAddress>{}</BindAddress></Config>' > /config/config.xml; sed -i 's|<Port>[^<]*</Port>|<Port>{}</Port>|g' /config/config.xml; sed -i 's|<BindAddress>[^<]*</BindAddress>|<BindAddress>{}</BindAddress>|g' /config/config.xml; nix run nixpkgs#{}",
                p, addr, p, addr, name.to_lowercase()
            )
        }
        "jellyfin" => {
            let mappings = port.as_ref().map(|s| parse_ports(s)).unwrap_or_default();
            
            // Default HTTP and HTTPS ports for Jellyfin
            let mut http_port = 8096;
            let mut https_port = 8920;
            
            for m in &mappings {
                if m.container == 8096 {
                    http_port = m.host;
                } else if m.container == 8920 {
                    https_port = m.host;
                }
            }
            
            let mut bind_opts = format!(" --port {}", http_port);
            if let Some(ref addr) = bind_address {
                if !addr.trim().is_empty() {
                    bind_opts.push_str(&format!(" --bind-to-address {}", addr));
                }
            }
            format!(
                "mkdir -p /config/config && [ ! -f /config/config/system.xml ] && echo '<ServerConfiguration><HttpsPortNumber>{}</HttpsPortNumber></ServerConfiguration>' > /config/config/system.xml; sed -i 's|<HttpsPortNumber>[^<]*</HttpsPortNumber>|<HttpsPortNumber>{}</HttpsPortNumber>|g' /config/config/system.xml; nix run nixpkgs#jellyfin -- --datadir /config/data --cachedir /config/cache --configdir /config/config{}",
                https_port, https_port, bind_opts
            )
        }
        "syncthing" => {
            let mappings = port.as_ref().map(|s| parse_ports(s)).unwrap_or_default();
            let mut gui_port = 8384;
            let mut sync_port = 22000;
            let mut local_ann_port = 21027;
            
            for m in &mappings {
                if m.container == 8384 {
                    gui_port = m.host;
                } else if m.container == 22000 {
                    sync_port = m.host;
                } else if m.container == 21027 {
                    local_ann_port = m.host;
                }
            }
            
            let addr = bind_address.clone().unwrap_or_else(|| "0.0.0.0".to_string());
            
            let mut patch_cmds = Vec::new();
            if sync_port != 22000 {
                patch_cmds.push(format!("sed -i 's|<listenAddress>tcp://:[^<]*</listenAddress>|<listenAddress>tcp://:{}</listenAddress>|g' /config/config.xml", sync_port));
                patch_cmds.push(format!("sed -i 's|<listenAddress>default</listenAddress>|<listenAddress>tcp://:{}</listenAddress>|g' /config/config.xml", sync_port));
            }
            if local_ann_port != 21027 {
                patch_cmds.push(format!("sed -i 's|<localAnnouncePort>[^<]*</localAnnouncePort>|<localAnnouncePort>{}</localAnnouncePort>|g' /config/config.xml", local_ann_port));
            }
            
            let patch_str = if patch_cmds.is_empty() {
                "".to_string()
            } else {
                format!(
                    "mkdir -p /config && [ -f /config/config.xml ] && {}; ",
                    patch_cmds.join(" && ")
                )
            };
            
            format!(
                "{}nix run nixpkgs#syncthing -- --home=/config --gui-address=http://{}:{}",
                patch_str, addr, gui_port
            )
        }
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
        extra_binds,
        port,
        bind_address,
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
            }),
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
        let cmd = get_service_command_preset("radarr", "/mnt/cache/appdata/radarr", "/mnt/user/media", 99, 100, false, Vec::new(), Some("7878".to_string()), Some("127.0.0.1".to_string())).unwrap();
        assert!(cmd.starts_with("exec unshare -m sh -c "));
        assert!(cmd.contains("mount -t tmpfs tmpfs /boot"));
        assert!(cmd.contains("mount --bind /mnt/cache/appdata/radarr /config"));
        assert!(cmd.contains("exec setpriv --reuid=99 --regid=100"));
        assert!(cmd.contains("nix run nixpkgs#radarr"));
        assert!(cmd.contains("sed -i 's|<Port>[^<]*</Port>|<Port>7878</Port>|g'"));

        let cmd_jellyfin = get_service_command_preset("jellyfin", "/mnt/cache/appdata/jellyfin", "-", 99, 100, false, Vec::new(), Some("8097:8096,8921:8920".to_string()), Some("10.0.0.5".to_string())).unwrap();
        assert!(cmd_jellyfin.contains("sed -i 's|<HttpsPortNumber>[^<]*</HttpsPortNumber>|<HttpsPortNumber>8921</HttpsPortNumber>|g'"));
        assert!(cmd_jellyfin.contains("--port 8097"));
        assert!(cmd_jellyfin.contains("--bind-to-address 10.0.0.5"));

        let err = get_service_command_preset("invalid", "/tmp", "/tmp", 99, 100, false, Vec::new(), None, None);
        assert!(err.is_err());
    }
}
