use crate::process;
use crate::search;
use crate::sandbox;
use crate::config;
use std::process::exit;

fn validate_name(name: &str) {
    if !crate::store::is_valid_service_name(name) {
        eprintln!("Error: Invalid service name.");
        exit(1);
    }
}

pub fn service_action(action: &str, name: &str) {
    validate_name(name);
    if let Err(e) = process::send_service_action(29704, name, action) {
        eprintln!("Service action failed: {}", e);
        exit(1);
    }
    println!("Action {} sent to service {}.", action, name);
}

pub fn autostart(name: &str, toggle: &str) {
    validate_name(name);
    let toggle_lower = toggle.to_lowercase();
    let restart_policy = if toggle_lower == "on" || toggle_lower == "true" || toggle_lower == "1" {
        "always".to_string()
    } else {
        "no".to_string()
    };

    let mut cfg = match config::load_config("/boot/config/plugins/nix/process-compose.yml") {
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

        if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
            eprintln!("Error saving config: {}", e);
            exit(1);
        }

        let _ = std::process::Command::new("sh")
            .args(["-c", ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml 2>&1"])
            .stdin(std::process::Stdio::null())
            .output();
        crate::store::log_event("INFO", &format!("Service '{}' autostart set to '{}'.", name, toggle));
        println!("Autostart updated successfully.");
    } else {
        eprintln!("Error: Service {} not found in configuration.", name);
        exit(1);
    }
}

pub fn remove_service(name: &str) {
    validate_name(name);

    let mut cfg = match config::load_config("/boot/config/plugins/nix/process-compose.yml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            exit(1);
        }
    };

    if cfg.processes.remove(name).is_some() {
        if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
            eprintln!("Error saving config: {}", e);
            exit(1);
        }

        let _ = process::send_service_action(29704, name, "stop");
        let _ = std::process::Command::new("sh")
            .args(["-c", ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml 2>&1"])
            .stdin(std::process::Stdio::null())
            .output();
        let _ = std::fs::remove_file(format!("/boot/config/plugins/nix/metadata/{}.json", name));
        crate::store::log_event("INFO", &format!("Service '{}' successfully removed.", name));
        println!("Service {} successfully removed.", name);
    } else {
        eprintln!("Error: Service {} not found in configuration.", name);
        exit(1);
    }
}

pub fn install(package: &str) {
    if let Err(e) = search::install_package(package) {
        crate::store::log_event("ERROR", &format!("CLI package installation failed for '{}': {}", package, e));
        eprintln!("Installation failed: {}", e);
        exit(1);
    }
    crate::store::log_event("INFO", &format!("CLI package '{}' successfully installed/added.", package));
    println!("Successfully installed package: {}", package);
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
        Ok(cmd) => println!("{}", cmd),
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
    validate_name(name);
    let media_val = if media == "-" { "" } else { media };
    let gpu = gpu_str == "1" || gpu_str == "true";
    let extra_binds = extra_binds_str
        .and_then(|s| if s != "-" && !s.is_empty() { sandbox::parse_binds_string(s).ok() } else { None })
        .unwrap_or_default();
    let port = port_str.and_then(|s| if s != "-" && !s.is_empty() { Some(s.to_string()) } else { None });
    let bind_address = bind_address_str.and_then(|s| if s != "-" && !s.is_empty() { Some(s.to_string()) } else { None });

    match config::get_service_command_preset(name, appdata, media_val, puid, pgid, gpu, None, extra_binds, port, bind_address) {
        Ok(cmd) => println!("{}", cmd),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

pub fn add_service(name: &str, cmd: &str, restart_policy: Option<&str>) {
    validate_name(name);
    let restart = restart_policy.unwrap_or("always").to_string();

    let mut cfg = match config::load_config("/boot/config/plugins/nix/process-compose.yml") {
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

    let log_location = Some(format!("/var/log/nix-services/{}.log", name));
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

    if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
        eprintln!("Error saving config: {}", e);
        exit(1);
    }
    println!("Service successfully added.");
}
