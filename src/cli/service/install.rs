use crate::search;
use std::process::exit;

pub fn install(package: &str) {
    if let Err(e) = search::install_package(package) {
        crate::store::log_event("ERROR", &format!("CLI package installation failed for '{package}': {e}"));
        exit(1);
    }
    crate::store::log_event("INFO", &format!("CLI package '{package}' successfully installed/added."));
    println!("Successfully installed package: {package}");
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
        crate::store::log_event("ERROR", &format!("Invalid service name '{name}' for preset"));
        exit(1);
    }
    let media_val = if media == "-" { "" } else { media };
    let gpu = gpu_str == "1" || gpu_str == "true";
    let extra_binds = extra_binds_str
        .and_then(|s| if s != "-" && !s.is_empty() { crate::sandbox::parse_binds_string(s).ok() } else { None })
        .unwrap_or_default();
    let port = port_str.and_then(|s| if s != "-" && !s.is_empty() { Some(s.to_string()) } else { None });
    let bind_address = bind_address_str.and_then(|s| if s != "-" && !s.is_empty() { Some(s.to_string()) } else { None });

    match crate::config::get_service_command_preset(name, appdata, media_val, puid, pgid, gpu, None, extra_binds, port, bind_address) {
        Ok(cmd) => println!("{cmd}"),
        Err(e) => {
            crate::store::log_event("ERROR", &format!("Failed to resolve preset command for '{name}': {e}"));
            exit(1);
        }
    }
}