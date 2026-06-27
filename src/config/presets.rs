use crate::sandbox::{build_bwrap_command, parse_ports, SandboxConfig};

/// Retrieves the command preset templates for common services.
/// Customizes directory paths and applies drop-privileges wrapper parameters.
pub fn get_service_command_preset(
    name: &str,
    appdata: &str,
    media: &str,
    puid: u32,
    pgid: u32,
    enable_gpu: bool,
    gpus: Option<String>,
    extra_binds: Vec<(String, String)>,
    port: Option<String>,
    bind_address: Option<String>,
) -> Result<String, String> {
    let media_path = if media.trim().is_empty() || media == "-" {
        None
    } else {
        Some(media.to_string())
    };

    let mut host_init_commands = Vec::new();

    let inner_command = match name.to_lowercase().as_str() {
        "radarr" | "sonarr" => {
            let default_port = if name.to_lowercase() == "radarr" { 7878 } else { 8989 };
            let mappings = port.as_ref().map(|s| parse_ports(s)).unwrap_or_default();
            let p = mappings.first().map(|m| m.host).unwrap_or(default_port);
            let addr = bind_address.clone().unwrap_or_else(|| "*".to_string());
            
            host_init_commands.push(format!("mkdir -p {}", appdata));
            host_init_commands.push(format!(
                "if [ ! -f {}/config.xml ]; then echo '<Config><Port>{}</Port><BindAddress>{}</BindAddress></Config>' > {}/config.xml; fi",
                appdata, p, addr, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<Port>[^<]*</Port>|<Port>{}</Port>|g' {}/config.xml",
                p, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<BindAddress>[^<]*</BindAddress>|<BindAddress>{}</BindAddress>|g' {}/config.xml",
                addr, appdata
            ));
            host_init_commands.push(format!("chown -R {}:{} {}", puid, pgid, appdata));

            format!("exec nix run nixpkgs#{}", name.to_lowercase())
        }
        "jellyfin" => {
            let mappings = port.as_ref().map(|s| parse_ports(s)).unwrap_or_default();
            let mut http_port = 8096;
            let mut https_port = 8920;
            
            for m in &mappings {
                if m.container == 8096 {
                    http_port = m.host;
                } else if m.container == 8920 {
                    https_port = m.host;
                }
            }
            
            let addr = bind_address.clone().unwrap_or_else(|| "0.0.0.0".to_string());
            
            host_init_commands.push(format!("mkdir -p {}/config", appdata));
            host_init_commands.push(format!(
                "if [ ! -f {}/config/network.xml ]; then echo '<?xml version=\"1.0\" encoding=\"utf-8\"?><NetworkConfiguration><LocalPortNumber>8096</LocalPortNumber><HttpsPortNumber>8920</HttpsPortNumber><EnableHttps>false</EnableHttps><PublicPort>8096</PublicPort><PublicHttpsPort>8920</PublicHttpsPort><BindToAddress>0.0.0.0</BindToAddress></NetworkConfiguration>' > {}/config/network.xml; fi",
                appdata, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<LocalPortNumber>[^<]*</LocalPortNumber>|<LocalPortNumber>{}</LocalPortNumber>|g' {}/config/network.xml",
                http_port, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<HttpsPortNumber>[^<]*</HttpsPortNumber>|<HttpsPortNumber>{}</HttpsPortNumber>|g' {}/config/network.xml",
                https_port, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<PublicPort>[^<]*</PublicPort>|<PublicPort>{}</PublicPort>|g' {}/config/network.xml",
                http_port, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<PublicHttpsPort>[^<]*</PublicHttpsPort>|<PublicHttpsPort>{}</PublicHttpsPort>|g' {}/config/network.xml",
                https_port, appdata
            ));
            host_init_commands.push(format!(
                "sed -i 's|<BindToAddress>[^<]*</BindToAddress>|<BindToAddress>{}</BindToAddress>|g' {}/config/network.xml",
                addr, appdata
            ));
            host_init_commands.push(format!("chown -R {}:{} {}", puid, pgid, appdata));

            format!("exec nix run nixpkgs#jellyfin -- --datadir /config/data --cachedir /config/cache --configdir /config/config")
        }
        "syncthing" => {
            let mappings = port.as_ref().map(|s| parse_ports(s)).unwrap_or_default();
            let mut gui_port = 8384;
            let mut sync_port = 22000;
            let mut local_ann_port = 21027;
            
            for m in &mappings {
                if m.container == 8384 {
                    gui_port = m.host;
                } else if m.container == 22000 {
                    sync_port = m.host;
                } else if m.container == 21027 {
                    local_ann_port = m.host;
                }
            }
            
            let addr = bind_address.clone().unwrap_or_else(|| "0.0.0.0".to_string());
            
            host_init_commands.push(format!("mkdir -p {}", appdata));
            if sync_port != 22000 {
                host_init_commands.push(format!(
                    "sed -i 's|<listenAddress>tcp://:[^<]*</listenAddress>|<listenAddress>tcp://:{}</listenAddress>|g' {}/config.xml",
                    sync_port, appdata
                ));
                host_init_commands.push(format!(
                    "sed -i 's|<listenAddress>default</listenAddress>|<listenAddress>tcp://:{}</listenAddress>|g' {}/config.xml",
                    sync_port, appdata
                ));
            }
            if local_ann_port != 21027 {
                host_init_commands.push(format!(
                    "sed -i 's|<localAnnouncePort>[^<]*</localAnnouncePort>|<localAnnouncePort>{}</localAnnouncePort>|g' {}/config.xml",
                    local_ann_port, appdata
                ));
            }
            host_init_commands.push(format!("chown -R {}:{} {}", puid, pgid, appdata));

            format!("exec nix run nixpkgs#syncthing -- --home=/config --gui-address=http://{}:{}", addr, gui_port)
        }
        _ => return Err(format!("Unknown preset template: {}", name)),
    };

    build_bwrap_command(&SandboxConfig {
        name: name.to_string(),
        appdata_path: appdata.to_string(),
        media_path,
        puid,
        pgid,
        enable_gpu,
        gpus,
        inner_command,
        extra_binds,
        port,
        bind_address,
        host_init_commands,
    })
}
