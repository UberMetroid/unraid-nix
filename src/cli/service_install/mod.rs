use crate::config;
use crate::sandbox;
use std::process::exit;

pub mod setup;
pub mod config_writer;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExtraBind {
    pub host: String,
    pub sandbox: String,
}

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
    let mut compile_locally = false;
    let mut command_override = String::new();
    let mut network_isolation = false;

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
            "--network-isolation" => {
                if i + 1 < args.len() && (args[i+1] == "1" || args[i+1] == "true" || args[i+1] == "yes") {
                    network_isolation = true;
                    i += 2;
                } else if i + 1 < args.len() && (args[i+1] == "0" || args[i+1] == "false" || args[i+1] == "no") {
                    network_isolation = false;
                    i += 2;
                } else {
                    network_isolation = true;
                    i += 1;
                }
            }
            "--compile-locally" => {
                compile_locally = true;
                i += 1;
            }
            "--command-override" => {
                if i + 1 >= args.len() { eprintln!("Error: Missing value for --command-override"); exit(1); }
                command_override = args[i+1].clone();
                i += 2;
            }
            _ => { eprintln!("Unknown install-service flag: {}", args[i]); exit(1); }
        }
    }

    let extra_binds = setup::parse_and_create_binds(&extra_binds_json, puid, pgid);
    setup::setup_appdata_dir(&appdata, puid, pgid);

    let mut name = uri.replace("nixpkgs#", "");
    if let Some(pos) = name.rfind('/') { name = name[pos + 1..].to_string(); }
    if let Some(pos) = name.rfind(':') { name = name[pos + 1..].to_string(); }
    if let Some(pos) = name.rfind('#') { name = name[pos + 1..].to_string(); }
    name = name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();

    setup::verify_port_conflicts(&name, &port);

    let mut binds_vec = Vec::new();
    for b in &extra_binds {
        let host = b.host.trim();
        let sandbox = b.sandbox.trim();
        if !host.is_empty() && !sandbox.is_empty() {
            binds_vec.push((host.to_string(), sandbox.to_string()));
        }
    }

    let name_lower = name.to_lowercase();
    let preset_path = crate::config::get_preset_path(&name_lower);
    let has_preset = std::path::Path::new(&preset_path).exists() || ["radarr", "sonarr", "jellyfin", "syncthing"].contains(&name_lower.as_str());

    let cmd = if !command_override.trim().is_empty() {
        match sandbox::build_bwrap_command(&sandbox::SandboxConfig {
            name: name.clone(),
            appdata_path: appdata.clone(),
            media_path: media.clone(),
            puid,
            pgid,
            enable_gpu: gpu,
            gpus: gpus.clone(),
            inner_command: command_override.clone(),
            extra_binds: binds_vec.clone(),
            port: port.clone(),
            bind_address: bind_address.clone(),
            host_init_commands: Vec::new(),
            enable_network_isolation: network_isolation,
        }) {
            Ok(c) => c,
            Err(e) => { eprintln!("Error building sandbox command: {}", e); exit(1); }
        }
    } else if has_preset {
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
            enable_network_isolation: network_isolation,
        }) {
            Ok(c) => c,
            Err(e) => { eprintln!("Error building sandbox command: {}", e); exit(1); }
        }
    };

    let cmd = if compile_locally {
        cmd.replace("nix run ", "nix run --option substituters \"\" ")
    } else {
        cmd
    };

    config_writer::write_config_and_metadata(
        &name,
        &uri,
        &appdata,
        puid,
        pgid,
        gpu,
        gpus.as_deref().unwrap_or(""),
        &extra_binds_json,
        port.as_deref().unwrap_or(""),
        bind_address.as_deref().unwrap_or(""),
        &env_vars_json,
        compile_locally,
        &command_override,
        network_isolation,
        cmd,
    );
}
