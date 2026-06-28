use crate::process;
use crate::config;
use crate::unraid::PROCESS_COMPOSE_CONFIG;
use std::process::{exit, Command};

const SCRIPT_RELOAD_SUPERVISOR: &str = "/usr/local/emhttp/plugins/nix/scripts/reload-supervisor.sh";

pub fn service_action(action: &str, name: &str) {
    if !crate::store::is_valid_service_name(name) {
        crate::store::log_event("ERROR", &format!("Invalid service name '{name}' for action '{action}'"));
        exit(1);
    }
    if let Err(e) = process::send_service_action(crate::unraid::SUPERVISOR_PORT, name, action) {
        crate::store::log_event("ERROR", &format!("Service action '{action}' failed for '{name}': {e}"));
        exit(1);
    }
    println!("Action {action} sent to service {name}.");
}

pub fn autostart(name: &str, toggle: &str) {
    if !crate::store::is_valid_service_name(name) {
        crate::store::log_event("ERROR", &format!("Invalid service name '{name}' for autostart"));
        exit(1);
    }
    let toggle_lower = toggle.to_lowercase();
    let restart_policy = if toggle_lower == "on" || toggle_lower == "true" || toggle_lower == "1" {
        "always".to_string()
    } else {
        "no".to_string()
    };

    let mut cfg = match config::load_config(PROCESS_COMPOSE_CONFIG) {
        Ok(c) => c,
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Failed to load process-compose config: {e}"));
            exit(1);
        }
    };

    if let Some(p) = cfg.processes.get_mut(name) {
        if let Some(ref mut a) = p.availability {
            a.restart = restart_policy;
        } else {
            p.availability = Some(config::Availability {
                restart: restart_policy,
                backoff_seconds: Some(5),
                max_restarts: None,
            });
        }

        if let Err(e) = config::save_config(&cfg, PROCESS_COMPOSE_CONFIG) {
            crate::store::log_event("ERROR", &format!("Failed to save process-compose config after autostart change for '{name}': {e}"));
            exit(1);
        }

        let _ = Command::new(SCRIPT_RELOAD_SUPERVISOR).status();
        crate::store::log_event("INFO", &format!("Service '{name}' autostart set to '{toggle}'."));
        println!("Autostart updated successfully.");
    } else {
        crate::store::log_event("ERROR", &format!("Service '{name}' not found in process-compose configuration"));
        exit(1);
    }
}

pub fn remove_service(name: &str) {
    if !crate::store::is_valid_service_name(name) {
        crate::store::log_event("ERROR", &format!("Invalid service name '{name}' for remove-service"));
        exit(1);
    }

    let mut cfg = match config::load_config(PROCESS_COMPOSE_CONFIG) {
        Ok(c) => c,
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Failed to load process-compose config: {e}"));
            exit(1);
        }
    };

    if cfg.processes.remove(name).is_some() {
        if let Err(e) = config::save_config(&cfg, PROCESS_COMPOSE_CONFIG) {
            crate::store::log_event("ERROR", &format!("Failed to save process-compose config after removing '{name}': {e}"));
            exit(1);
        }

        let _ = process::send_service_action(crate::unraid::SUPERVISOR_PORT, name, "stop");
        let _ = Command::new(SCRIPT_RELOAD_SUPERVISOR).status();
        let _ = std::fs::remove_file(format!("{}/{name}.json", crate::unraid::METADATA_DIR));
        crate::store::log_event("INFO", &format!("Service '{name}' successfully removed."));
        println!("Service {name} successfully removed.");
    } else {
        crate::store::log_event("ERROR", &format!("Service '{name}' not found in process-compose configuration"));
        exit(1);
    }
}

pub fn add_service(name: &str, cmd: &str, restart_policy: Option<&str>) {
    if !crate::store::is_valid_service_name(name) {
        crate::store::log_event("ERROR", &format!("Invalid service name '{name}' for add-service"));
        exit(1);
    }
    let restart = restart_policy.unwrap_or("always").to_string();

    let mut cfg = match config::load_config(PROCESS_COMPOSE_CONFIG) {
        Ok(c) => c,
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Failed to load process-compose config: {e}"));
            exit(1);
        }
    };

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

    let log_location = Some(format!("/var/log/nix-services/{name}.log"));
    cfg.processes.insert(name.to_string(), config::ProcessDefinition {
        command: cmd.to_string(),
        availability: Some(config::Availability {
            restart,
            backoff_seconds: Some(5),
            max_restarts: None,
        }),
        environment: None,
        log_location,
        log_configuration: None,
    });

    if let Err(e) = config::save_config(&cfg, PROCESS_COMPOSE_CONFIG) {
        crate::store::log_event("ERROR", &format!("Failed to save process-compose config after adding '{name}': {e}"));
        exit(1);
    }
    println!("Service successfully added.");
}