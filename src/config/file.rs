use crate::config::{LogConfiguration, ProcessComposeConfig};
use std::fs;

/// Loads the process-compose configuration from the specified file path.
pub fn load_config(file_path: &str) -> Result<ProcessComposeConfig, String> {
    if fs::metadata(file_path).is_err() {
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
                rotation: None,
            }),
            processes: std::collections::HashMap::new(),
        });
    }

    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read config file: {}", e))?;

    crate::config::yaml::parse_config(&content).map_err(|e| format!("Failed to parse YAML: {}", e))
}

/// Saves the process-compose configuration back to the specified file path.
pub fn save_config(config: &ProcessComposeConfig, file_path: &str) -> Result<(), String> {
    let yaml = crate::config::yaml::serialize_config(config);

    fs::write(file_path, yaml).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
