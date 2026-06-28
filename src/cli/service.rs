use crate::process;
use crate::search;
use crate::sandbox;
use crate::config;
use crate::unraid::{METADATA_DIR, PROCESS_COMPOSE_CONFIG, SUPERVISOR_PORT};
use std::process::{exit, Command};

const SCRIPT_RELOAD_SUPERVISOR: &str = "/usr/local/emhttp/plugins/nix/scripts/reload-supervisor.sh";

pub fn service_action(action: &str, name: &str) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
    if let Err(e) = process::send_service_action(SUPERVISOR_PORT, name, action) {
        eprintln!("Service action failed: {}", e);
        exit(1);
    }
    println!("Action {action} sent to service {name}.");
}

pub fn autostart(name: &str, toggle: &str) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
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
            eprintln!("Error loading config: {}", e);
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
            eprintln!("Error saving config: {}", e);
            exit(1);
        }

        let _ = Command::new(SCRIPT_RELOAD_SUPERVISOR).status();
        crate::store::log_event("INFO", &format!("Service '{name}' autostart set to '{toggle}'."));
        println!("Autostart updated successfully.");
    } else {
        eprintln!("Error: Service {name} not found in configuration.");
        exit(1);
    }
}

pub fn remove_service(name: &str) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }

    let mut cfg = match config::load_config(PROCESS_COMPOSE_CONFIG) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            exit(1);
        }
    };

    if cfg.processes.remove(name).is_some() {
        if let Err(e) = config::save_config(&cfg, PROCESS_COMPOSE_CONFIG) {
            eprintln!("Error saving config: {}", e);
            exit(1);
        }

        let _ = process::send_service_action(SUPERVISOR_PORT, name, "stop");
        let _ = Command::new(SCRIPT_RELOAD_SUPERVISOR).status();
        let _ = std::fs::remove_file(format!("{METADATA_DIR}/{name}.json"));
        crate::store::log_event("INFO", &format!("Service '{name}' successfully removed."));
        println!("Service {name} successfully removed.");
    } else {
        eprintln!("Error: Service {name} not found in configuration.");
        exit(1);
    }
}

pub fn install(package: &str) {
    if let Err(e) = search::install_package(package) {
        crate::store::log_event("ERROR", &format!("CLI package installation failed for '{package}': {e}"));
        eprintln!("Installation failed: {}", e);
        exit(1);
    }
    crate::store::log_event("INFO", &format!("CLI package '{package}' successfully installed/added."));
    println!("Successfully installed package: {package}");
}

pub fn sandbox_cmd(args: &crate::cli::args::SandboxArgs) {
    let config = sandbox::SandboxConfig {
        name: args.name.clone(),
        appdata_path: args.appdata.clone(),
        media_path: args.media.clone(),
        puid: args.puid,
        pgid: args.pgid,
        enable_gpu: args.gpu,
        gpus: args.gpus.clone(),
        inner_command: args.cmd.clone(),
        extra_binds: args.extra_binds.as_ref()
            .and_then(|s| sandbox::parse_binds_string(s).ok())
            .unwrap_or_default(),
        port: args.port.clone(),
        bind_address: args.bind_address.clone(),
        host_init_commands: Vec::new(),
        enable_network_isolation: args.network_isolation,
    };
    match sandbox::build_bwrap_command(&config) {
        Ok(cmd) => println!("{cmd}"),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

pub fn preset_cmd(
    name: &str,
    appdata: &str,
    media: &str,
    puid: u32,
    pgid: u32,
    gpu_str: &str,
    extra_binds_str: Option<&str>,
    port_str: Option<&str>,
    bind_address_str: Option<&str>,
) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
    let media_val = if media == "-" { "" } else { media };
    let gpu = gpu_str == "1" || gpu_str == "true";
    let extra_binds = extra_binds_str
        .and_then(|s| if s != "-" && !s.is_empty() { sandbox::parse_binds_string(s).ok() } else { None })
        .unwrap_or_default();
    let port = port_str.and_then(|s| if s != "-" && !s.is_empty() { Some(s.to_string()) } else { None });
    let bind_address = bind_address_str.and_then(|s| if s != "-" && !s.is_empty() { Some(s.to_string()) } else { None });

    match config::get_service_command_preset(name, appdata, media_val, puid, pgid, gpu, None, extra_binds, port, bind_address) {
        Ok(cmd) => println!("{cmd}"),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

pub fn add_service(name: &str, cmd: &str, restart_policy: Option<&str>) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
    let restart = restart_policy.unwrap_or("always").to_string();

    let mut cfg = match config::load_config(PROCESS_COMPOSE_CONFIG) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
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
        eprintln!("Error saving config: {}", e);
        exit(1);
    }
    println!("Service successfully added.");
}
