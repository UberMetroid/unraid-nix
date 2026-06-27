use crate::process;
use crate::search;
use crate::sandbox;
use crate::config;
use std::process::exit;

fn validate_name(name: &str) {
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        eprintln!("Error: Invalid service name. Must be alphanumeric with dashes or underscores only.");
        exit(1);
    }
}

pub fn service_action(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Error: Missing action or service name.");
        exit(1);
    }
    let action = &args[2];
    let name = &args[3];
    validate_name(name);
    if let Err(e) = process::send_service_action(29704, name, action) {
        eprintln!("Service action failed: {}", e);
        exit(1);
    }
    println!("Action {} sent to service {}.", action, name);
}

pub fn autostart(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Error: Missing service name or toggle value (on/off).");
        exit(1);
    }
    let name = &args[2];
    validate_name(name);
    let toggle = args[3].to_lowercase();
    let restart_policy = if toggle == "on" || toggle == "true" || toggle == "1" {
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
            .args(&["-c", ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml 2>&1"])
            .output();
        println!("Autostart updated successfully.");
    } else {
        eprintln!("Error: Service {} not found in configuration.", name);
        exit(1);
    }
}

pub fn remove_service(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Missing service name.");
        exit(1);
    }
    let name = &args[2];
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
            .args(&["-c", ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 project update -f /boot/config/plugins/nix/process-compose.yml 2>&1"])
            .output();
        let _ = std::fs::remove_file(format!("/boot/config/plugins/nix/metadata/{}.json", name));
        println!("Service {} successfully removed.", name);
    } else {
        eprintln!("Error: Service {} not found in configuration.", name);
        exit(1);
    }
}

pub fn install(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Missing package name.");
        exit(1);
    }
    let package = &args[2];
    if let Err(e) = search::install_package(package) {
        eprintln!("Installation failed: {}", e);
        exit(1);
    }
    println!("Successfully installed package: {}", package);
}

pub fn sandbox_cmd(args: &[String]) {
    match sandbox::parse_sandbox_args(args) {
        Ok(cmd) => println!("{}", cmd),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

pub fn preset_cmd(args: &[String]) {
    if args.len() < 8 {
        eprintln!("Error: Missing arguments: preset <name> <appdata> <media> <puid> <pgid> <gpu> [extra_binds] [port] [bind_address]");
        exit(1);
    }
    let name = &args[2];
    validate_name(name);
    let appdata = &args[3];
    let media = if args[4] == "-" { "" } else { &args[4] };
    let puid = args[5].parse::<u32>().unwrap_or(99);
    let pgid = args[6].parse::<u32>().unwrap_or(100);
    let gpu = args[7] == "1" || args[7] == "true";
    let extra_binds = if args.len() >= 9 && args[8] != "-" && !args[8].is_empty() {
        sandbox::parse_binds_string(&args[8]).unwrap_or_default()
    } else {
        Vec::new()
    };
    let port = if args.len() >= 10 && args[9] != "-" && !args[9].is_empty() {
        Some(args[9].clone())
    } else {
        None
    };
    let bind_address = if args.len() >= 11 && args[10] != "-" && !args[10].is_empty() {
        Some(args[10].clone())
    } else {
        None
    };

    match config::get_service_command_preset(name, appdata, media, puid, pgid, gpu, None, extra_binds, port, bind_address) {
        Ok(cmd) => println!("{}", cmd),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

pub fn add_service(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Error: Missing arguments: add-service <name> <command> [restart_policy]");
        exit(1);
    }
    let name = args[2].clone();
    validate_name(&name);
    let cmd = args[3].clone();
    let restart = if args.len() >= 5 { args[4].clone() } else { "always".to_string() };

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
        });
    }

    let log_location = Some(format!("/var/log/nix-services/{}.log", name));
    cfg.processes.insert(name, config::ProcessDefinition {
        command: cmd,
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
