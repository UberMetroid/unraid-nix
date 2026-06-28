use crate::config;
use crate::unraid::{METADATA_DIR, PROCESS_COMPOSE_CONFIG};
use std::process::exit;

pub fn write_config_and_metadata(
    name: &str,
    uri: &str,
    appdata: &str,
    puid: u32,
    pgid: u32,
    gpu: bool,
    gpus: &str,
    extra_binds_json: &str,
    port: &str,
    bind_address: &str,
    env_vars_json: &str,
    compile_locally: bool,
    command_override: &str,
    network_isolation: bool,
    cmd: String,
) {
    let mut cfg = config::load_config(PROCESS_COMPOSE_CONFIG).unwrap_or_else(|_| {
        config::ProcessComposeConfig {
            version: "0.5".to_string(),
            environment: None,
            log_configuration: None,
            processes: std::collections::HashMap::new(),
        }
    });
    if cfg.log_configuration.is_none() {
        cfg.log_configuration = Some(config::LogConfiguration {
            add_timestamp: Some(true),
            fields_order: Some(vec!["time".to_string(), "level".to_string(), "message".to_string()]),
            rotation: Some(config::Rotation {
                max_size_mb: Some(10),
                max_backups: Some(3),
                compress: Some(true),
            }),
        });
    } else if let Some(ref mut log_cfg) = cfg.log_configuration {
        if log_cfg.rotation.is_none() {
            log_cfg.rotation = Some(config::Rotation {
                max_size_mb: Some(10),
                max_backups: Some(3),
                compress: Some(true),
            });
        }
    }

    let mut env_list = Vec::new();
    if compile_locally {
        env_list.push("NIX_ENFORCE_NO_NATIVE=0".to_string());
        env_list.push("NIX_CFLAGS_COMPILE=-march=native".to_string());
        env_list.push("RUSTFLAGS=-C target-cpu=native".to_string());
    }
    if !env_vars_json.is_empty() {
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(env_vars_json) {
            for (k, v) in map {
                if let Some(val_str) = v.as_str() {
                    env_list.push(format!("{k}={val_str}"));
                } else {
                    env_list.push(format!("{k}={v}"));
                }
            }
        }
    }
    let env_opt = if env_list.is_empty() { None } else { Some(env_list) };

    let log_location = Some(format!("/var/log/nix-services/{name}.log"));
    cfg.processes.insert(name.to_string(), config::ProcessDefinition {
        command: cmd,
        availability: Some(config::Availability {
            restart: "always".to_string(),
            backoff_seconds: Some(5),
            max_restarts: None,
        }),
        environment: env_opt,
        log_location,
        log_configuration: None,
    });

    if let Err(e) = config::save_config(&cfg, PROCESS_COMPOSE_CONFIG) {
        eprintln!("Error saving config: {}", e);
        exit(1);
    }

    let metadata = serde_json::json!({
        "name": name,
        "uri": uri,
        "appdata": appdata,
        "puid": puid,
        "pgid": pgid,
        "gpu": if gpu { "1" } else { "0" },
        "gpus": gpus,
        "extra_binds": extra_binds_json,
        "port": port,
        "bind_address": bind_address,
        "env_vars": env_vars_json,
        "compile_locally": if compile_locally { "1" } else { "0" },
        "command_override": command_override,
        "network_isolation": if network_isolation { "1" } else { "0" },
    });
    if let Err(e) = std::fs::create_dir_all(METADATA_DIR) {
        crate::store::log_event("ERROR", &format!("Failed to create metadata dir '{METADATA_DIR}': {e}"));
        eprintln!("Error: failed to create metadata dir: {e}");
        exit(1);
    }
    let body = match serde_json::to_string_pretty(&metadata) {
        Ok(s) => s,
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Failed to serialize metadata for '{name}': {e}"));
            eprintln!("Error: failed to serialize metadata: {e}");
            exit(1);
        }
    };
    if let Err(e) = std::fs::write(format!("{METADATA_DIR}/{name}.json"), body) {
        crate::store::log_event("ERROR", &format!("Failed to write metadata file for '{name}': {e}"));
        eprintln!("Error: failed to write metadata: {e}");
        exit(1);
    }

    if let Err(e) = crate::cli::supervisor::restart_nix_supervisor() {
        crate::store::log_event("ERROR", &format!("Failed to restart supervisor after installing service '{name}': {e}"));
        eprintln!("Error restarting supervisor: {e}");
        exit(1);
    }
    crate::store::log_event("INFO", &format!("Service '{name}' installed/updated successfully. URI: {uri}"));
    println!("Service successfully installed.");
}
