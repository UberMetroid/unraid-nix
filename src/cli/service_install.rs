use crate::config;
use crate::sandbox;
use std::process::exit;

/// Struct for parsing user-defined extra shared paths (from JSON formatted payloads).
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct ExtraBind {
    host: String,
    sandbox: String,
}

/// CLI handler command to install, generate, and configure background services.
/// Parses configuration options, creates necessary host mount points,
/// configures the process supervisor configuration, creates instance metadata,
/// and reloads the active process-compose daemon.
pub fn install_service(args: &[String]) {
    let mut uri = String::new();
    let mut appdata = String::new();
    let mut media = None;
    let mut puid = 99;
    let mut pgid = 100;
    let mut gpu = false;
    let mut gpus = None;
    let mut extra_binds_json = String::new();
    let mut port = None;
    let mut bind_address = None;
    let mut env_vars_json = String::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--uri" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --uri"); exit(1); }
                uri = args[i+1].clone();
                i += 2;
            }
            "--appdata" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --appdata"); exit(1); }
                appdata = args[i+1].clone();
                i += 2;
            }
            "--media" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --media"); exit(1); }
                let val = args[i+1].clone();
                media = if val.trim().is_empty() || val == "-" { None } else { Some(val) };
                i += 2;
            }
            "--puid" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --puid"); exit(1); }
                puid = args[i+1].parse::<u32>().unwrap_or(99);
                i += 2;
            }
            "--pgid" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --pgid"); exit(1); }
                pgid = args[i+1].parse::<u32>().unwrap_or(100);
                i += 2;
            }
            "--gpu" => {
                if i + 1 < args.len() && (args[i+1] == "1" || args[i+1] == "true") {
                    gpu = true;
                    i += 2;
                } else if i + 1 < args.len() && (args[i+1] == "0" || args[i+1] == "false") {
                    gpu = false;
                    i += 2;
                } else {
                    gpu = true;
                    i += 1;
                }
            }
            "--extra-binds" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --extra-binds"); exit(1); }
                extra_binds_json = args[i+1].clone();
                i += 2;
            }
            "--port" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --port"); exit(1); }
                let val = args[i+1].clone();
                port = if val.trim().is_empty() || val == "-" { None } else { Some(val) };
                i += 2;
            }
            "--bind-address" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --bind-address"); exit(1); }
                let val = args[i+1].clone();
                bind_address = if val.trim().is_empty() || val == "-" { None } else { Some(val) };
                i += 2;
            }
            "--gpus" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --gpus"); exit(1); }
                let val = args[i+1].clone();
                gpus = if val.trim().is_empty() || val == "-" { None } else { Some(val) };
                i += 2;
            }
            "--env-vars" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --env-vars"); exit(1); }
                env_vars_json = args[i+1].clone();
                i += 2;
            }
            _ => { eprintln!("Unknown install-service flag: {}", args[i]); exit(1); }
        }
    }

    // 1. Create the primary appdata directory with proper user ownership
    if !appdata.is_empty() {
        let path = std::path::Path::new(&appdata);
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
            #[cfg(unix)]
            let _ = std::os::unix::fs::chown(path, Some(puid), Some(pgid));
        }
    }

    // 2. Parse and create extra mount paths defined by the user
    let extra_binds: Vec<ExtraBind> = if !extra_binds_json.is_empty() {
        serde_json::from_str(&extra_binds_json).unwrap_or_default()
    } else {
        Vec::new()
    };

    for b in &extra_binds {
        let host_path = b.host.trim();
        if !host_path.is_empty() {
            let path = std::path::Path::new(host_path);
            if !path.exists() {
                let _ = std::fs::create_dir_all(path);
                #[cfg(unix)]
                let _ = std::os::unix::fs::chown(path, Some(puid), Some(pgid));
            }
        }
    }

    // 3. Extract the service name from URI (e.g. "nixpkgs#radarr" -> "radarr")
    let mut name = uri.replace("nixpkgs#", "");
    if let Some(pos) = name.rfind('/') { name = name[pos + 1..].to_string(); }
    if let Some(pos) = name.rfind(':') { name = name[pos + 1..].to_string(); }
    if let Some(pos) = name.rfind('#') { name = name[pos + 1..].to_string(); }
    name = name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();

    // Pre-flight port conflict verification
    let check_port = if let Some(ref p_str) = port {
        p_str.parse::<u16>().ok()
    } else {
        let name_lower = name.to_lowercase();
        let preset_path = format!("/usr/local/emhttp/plugins/nix/presets/{}.json", name_lower);
        if std::path::Path::new(&preset_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&preset_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ports_arr) = json.get("default_ports").and_then(|p| p.as_array()) {
                        if !ports_arr.is_empty() {
                            ports_arr[0].get("host").and_then(|hp| hp.as_u64()).map(|p| p as u16)
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    };

    if let Some(p) = check_port {
        if crate::process::ports::is_port_in_use(p) {
            println!("[WARNING] Port {} is already bound by another service or Docker container on the host. This service may fail to start unless you configure a custom Port Override.", p);
        }
    }

    let mut binds_vec = Vec::new();
    for b in &extra_binds {
        let host = b.host.trim();
        let sandbox = b.sandbox.trim();
        if !host.is_empty() && !sandbox.is_empty() {
            binds_vec.push((host.to_string(), sandbox.to_string()));
        }
    }

    // 4. Load presets for known servers or fallback to custom sandbox builder
    let name_lower = name.to_lowercase();
    let preset_path = format!("/usr/local/emhttp/plugins/nix/presets/{}.json", name_lower);
    let has_preset = std::path::Path::new(&preset_path).exists() || ["radarr", "sonarr", "jellyfin", "syncthing"].contains(&name_lower.as_str());

    let cmd = if has_preset {
        match config::get_service_command_preset(
            &name,
            &appdata,
            media.as_deref().unwrap_or("-"),
            puid,
            pgid,
            gpu,
            gpus.clone(),
            binds_vec.clone(),
            port.clone(),
            bind_address.clone()
        ) {
            Ok(c) => c,
            Err(e) => { eprintln!("Error resolving preset: {}", e); exit(1); }
        }
    } else {
        match sandbox::build_bwrap_command(&sandbox::SandboxConfig {
            name: name.clone(),
            appdata_path: appdata.clone(),
            media_path: media.clone(),
            puid,
            pgid,
            enable_gpu: gpu,
            gpus: gpus.clone(),
            inner_command: format!("nix run {}", uri),
            extra_binds: binds_vec.clone(),
            port: port.clone(),
            bind_address: bind_address.clone(),
            host_init_commands: Vec::new(),
        }) {
            Ok(c) => c,
            Err(e) => { eprintln!("Error building sandbox command: {}", e); exit(1); }
        }
    };

    // 5. Update the process supervisor configuration yml
    let mut cfg = config::load_config("/boot/config/plugins/nix/process-compose.yml").unwrap_or_else(|_| {
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
    if !env_vars_json.is_empty() {
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&env_vars_json) {
            for (k, v) in map {
                if let Some(val_str) = v.as_str() {
                    env_list.push(format!("{}={}", k, val_str));
                } else {
                    env_list.push(format!("{}={}", k, v));
                }
            }
        }
    }
    let env_opt = if env_list.is_empty() { None } else { Some(env_list) };

    let log_location = Some(format!("/var/log/nix-services/{}.log", name));
    cfg.processes.insert(name.clone(), config::ProcessDefinition {
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

    if let Err(e) = config::save_config(&cfg, "/boot/config/plugins/nix/process-compose.yml") {
        eprintln!("Error saving config: {}", e);
        exit(1);
    }

    // 6. Write service metadata schema to configuration directory
    let metadata = serde_json::json!({
        "name": name,
        "uri": uri,
        "appdata": appdata,
        "puid": puid,
        "pgid": pgid,
        "gpu": if gpu { "1" } else { "0" },
        "gpus": gpus.as_deref().unwrap_or(""),
        "extra_binds": extra_binds_json,
        "port": port.unwrap_or_default(),
        "bind_address": bind_address.unwrap_or_default(),
        "env_vars": env_vars_json,
    });
    let meta_dir = "/boot/config/plugins/nix/metadata";
    let _ = std::fs::create_dir_all(meta_dir);
    let _ = std::fs::write(format!("{}/{}.json", meta_dir, name), serde_json::to_string_pretty(&metadata).unwrap());

    // 7. Restart supervisor daemon to load and start the new service definition
    if let Err(e) = super::supervisor::restart_nix_supervisor() {
        eprintln!("Error restarting supervisor: {}", e);
        exit(1);
    }
    println!("Service successfully installed.");
}
