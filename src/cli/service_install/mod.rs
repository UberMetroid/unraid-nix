use crate::config;
use crate::sandbox;
use std::process::exit;

pub mod config_writer;
pub mod setup;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExtraBind {
    pub host: String,
    pub sandbox: String,
}

pub fn install_service(args: &crate::cli::args::InstallServiceArgs) {
    let uri = &args.uri;
    let appdata = &args.appdata;
    let media = &args.media;
    let puid = args.puid;
    let pgid = args.pgid;
    let gpu = args.gpu;
    let gpus = &args.gpus;
    let extra_binds_json = args.extra_binds.as_deref().unwrap_or("");
    let port = &args.port;
    let bind_address = &args.bind_address;
    let env_vars_json = args.env_vars.as_deref().unwrap_or("");
    let compile_locally = args.compile_locally;
    let command_override = args.command_override.as_deref().unwrap_or("");
    let network_isolation = args.network_isolation;

    let extra_binds = setup::parse_and_create_binds(extra_binds_json, puid, pgid);
    setup::setup_appdata_dir(appdata, puid, pgid);

    let mut name = uri.replace("nixpkgs#", "");
    if let Some(pos) = name.rfind('/') {
        name = name[pos + 1..].to_string();
    }
    if let Some(pos) = name.rfind(':') {
        name = name[pos + 1..].to_string();
    }
    if let Some(pos) = name.rfind('#') {
        name = name[pos + 1..].to_string();
    }

    if !crate::store::is_valid_service_name(&name) {
        crate::store::log_event(
            "ERROR",
            &format!("Derived service name '{name}' from uri '{uri}' is invalid"),
        );
        exit(1);
    }

    setup::verify_port_conflicts(&name, port);

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
    let has_preset = std::path::Path::new(&preset_path).exists()
        || ["radarr", "sonarr", "jellyfin", "syncthing"].contains(&name_lower.as_str());

    let cmd = if !command_override.trim().is_empty() {
        match sandbox::build_bwrap_command(&sandbox::SandboxConfig {
            name: name.clone(),
            appdata_path: appdata.clone(),
            media_path: media.clone(),
            puid,
            pgid,
            enable_gpu: gpu,
            gpus: gpus.clone(),
            inner_command: command_override.to_string(),
            extra_binds: binds_vec.clone(),
            port: port.clone(),
            bind_address: bind_address.clone(),
            host_init_commands: Vec::new(),
            enable_network_isolation: network_isolation,
        }) {
            Ok(c) => c,
            Err(e) => {
                crate::store::log_event("ERROR", &format!("install-service: failed to build bubblewrap sandbox for '{name}' (override): {e}"));
                exit(1);
            }
        }
    } else if has_preset {
        match config::get_service_command_preset(
            &name,
            appdata,
            media.as_deref().unwrap_or("-"),
            puid,
            pgid,
            gpu,
            gpus.clone(),
            binds_vec.clone(),
            port.clone(),
            bind_address.clone(),
        ) {
            Ok(c) => c,
            Err(e) => {
                crate::store::log_event(
                    "ERROR",
                    &format!("install-service: failed to resolve preset for '{name}': {e}"),
                );
                exit(1);
            }
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
            inner_command: format!("nix run {uri}"),
            extra_binds: binds_vec.clone(),
            port: port.clone(),
            bind_address: bind_address.clone(),
            host_init_commands: Vec::new(),
            enable_network_isolation: network_isolation,
        }) {
            Ok(c) => c,
            Err(e) => {
                crate::store::log_event(
                    "ERROR",
                    &format!(
                        "install-service: failed to build bubblewrap sandbox for '{name}': {e}"
                    ),
                );
                exit(1);
            }
        }
    };

    let cmd = if compile_locally {
        cmd.replace("nix run ", "nix run --option substituters \"\" ")
    } else {
        cmd
    };

    config_writer::write_config_and_metadata(
        &name,
        uri,
        appdata,
        puid,
        pgid,
        gpu,
        gpus.as_deref().unwrap_or(""),
        extra_binds_json,
        port.as_deref().unwrap_or(""),
        bind_address.as_deref().unwrap_or(""),
        env_vars_json,
        compile_locally,
        command_override,
        network_isolation,
        cmd,
    );
}
