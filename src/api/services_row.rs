use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::{HostAddr, get_service_web_port, get_service_appdata_path, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};

struct FaIconConfig {
    icon: String,
    color: &'static str,
    bg: &'static str,
    border: &'static str,
}

struct StaticConfig {
    icon: &'static str,
    color: &'static str,
    bg: &'static str,
    border: &'static str,
}

fn get_static_config(name_lower: &str) -> StaticConfig {
    
    // AI & LLMs (Indigo/Cyberpunk Purple-Blue)
    if name_lower.contains("ollama") || name_lower.contains("open-webui") || name_lower.contains("localai") ||
       name_lower.contains("anythingllm") || name_lower.contains("librechat") || name_lower.contains("flowise") ||
       name_lower.contains("stable-diffusion") || name_lower.contains("comfyui") || name_lower.contains("text-generation-webui") ||
       name_lower.contains("invokeai") || name_lower.contains("fooocus") || name_lower.contains("dify") || name_lower.contains("lobe-chat") {
        return StaticConfig {
            icon: "fa-magic",
            color: "#a29bfe",
            bg: "rgba(162, 155, 254, 0.08)",
            border: "rgba(162, 155, 254, 0.2)",
        };
    }

    // Dashboards (Amber/Gold)
    if name_lower.contains("homepage") || name_lower.contains("homarr") || name_lower.contains("heimdall") ||
       name_lower.contains("dashy") || name_lower.contains("flame") || name_lower.contains("organizr") {
        return StaticConfig {
            icon: "fa-tachometer",
            color: "#fdcb6e",
            bg: "rgba(253, 203, 110, 0.08)",
            border: "rgba(253, 203, 110, 0.2)",
        };
    }

    // Gaming (Coral/Red)
    if name_lower.contains("emulatorjs") || name_lower.contains("romm") || name_lower.contains("pterodactyl") ||
       name_lower.contains("pufferpanel") || name_lower.contains("lancache") || name_lower.contains("minecraft-server") ||
       name_lower.contains("steamcmd") {
        return StaticConfig {
            icon: "fa-gamepad",
            color: "#ff7675",
            bg: "rgba(255, 118, 117, 0.08)",
            border: "rgba(255, 118, 117, 0.2)",
        };
    }

    // Productivity (Emerald Green)
    if name_lower.contains("paperless-ngx") || name_lower.contains("stirling-pdf") || name_lower.contains("joplin-server") ||
       name_lower.contains("trilium-notes") || name_lower.contains("bookstack") || name_lower.contains("wiki-js") ||
       name_lower.contains("dokuwiki") || name_lower.contains("mediawiki") || name_lower.contains("wekan") ||
       name_lower.contains("focalboard") || name_lower.contains("leantime") || name_lower.contains("kanboard") ||
       name_lower.contains("etherpad") || name_lower.contains("hedgedoc") || name_lower.contains("outline") ||
       name_lower.contains("excalidraw") {
        return StaticConfig {
            icon: "fa-file-text-o",
            color: "#2ecc71",
            bg: "rgba(46, 204, 113, 0.08)",
            border: "rgba(46, 204, 113, 0.2)",
        };
    }

    // Media Players (Cyan/Blue)
    if name_lower.contains("jellyfin") || name_lower.contains("plex") || name_lower.contains("emby") ||
       name_lower.contains("navidrome") || name_lower.contains("airsonic") || name_lower.contains("subsonic") || 
       name_lower.contains("lidarr") || name_lower.contains("audiobookshelf") || name_lower.contains("kavita") || 
       name_lower.contains("calibre") {
        let icon = if name_lower.contains("jellyfin") || name_lower.contains("plex") || name_lower.contains("emby") {
            "fa-play-circle"
        } else if name_lower.contains("navidrome") || name_lower.contains("airsonic") || name_lower.contains("subsonic") || name_lower.contains("lidarr") {
            "fa-music"
        } else if name_lower.contains("calibre") || name_lower.contains("audiobookshelf") || name_lower.contains("kavita") {
            "fa-book"
        } else {
            "fa-music"
        };
        return StaticConfig {
            icon,
            color: "#00a1ff",
            bg: "rgba(0, 161, 255, 0.08)",
            border: "rgba(0, 161, 255, 0.2)",
        };
    }
    
    // Servarr (Orange)
    if name_lower.contains("sonarr") || name_lower.contains("sickrage") || name_lower.contains("sickchill") ||
       name_lower.contains("radarr") || name_lower.contains("couchpotato") || name_lower.contains("readarr") ||
       name_lower.contains("bazarr") || name_lower.contains("prowlarr") || name_lower.contains("jackett") {
        let icon = if name_lower.contains("sonarr") || name_lower.contains("sickrage") || name_lower.contains("sickchill") {
            "fa-television"
        } else if name_lower.contains("radarr") || name_lower.contains("couchpotato") {
            "fa-film"
        } else if name_lower.contains("readarr") {
            "fa-book"
        } else if name_lower.contains("bazarr") {
            "fa-closed-captioning"
        } else {
            "fa-search"
        };
        return StaticConfig {
            icon,
            color: "#e67e22",
            bg: "rgba(230, 126, 34, 0.08)",
            border: "rgba(230, 126, 34, 0.2)",
        };
    }

    // General Automation & Workflows (Pink/Magenta)
    if name_lower.contains("n8n") || name_lower.contains("node-red") || name_lower.contains("nodered") ||
       name_lower.contains("changedetection") || name_lower.contains("apprise") || name_lower.contains("gotify") ||
       name_lower.contains("ntfy") || name_lower.contains("huginn") || name_lower.contains("activepieces") {
        let icon = if name_lower.contains("n8n") {
            "fa-sitemap"
        } else if name_lower.contains("node-red") || name_lower.contains("nodered") {
            "fa-code-fork"
        } else {
            "fa-refresh"
        };
        return StaticConfig {
            icon,
            color: "#e84393",
            bg: "rgba(232, 67, 147, 0.08)",
            border: "rgba(232, 67, 147, 0.2)",
        };
    }

    // Download Clients (Green)
    if name_lower.contains("transmission") || name_lower.contains("sabnzbd") || name_lower.contains("nzbget") || 
       name_lower.contains("qbittorrent") || name_lower.contains("deluge") || name_lower.contains("rtorrent") || name_lower.contains("aria2") {
        return StaticConfig {
            icon: "fa-download",
            color: "#2ecc71",
            bg: "rgba(46, 204, 113, 0.08)",
            border: "rgba(46, 204, 113, 0.2)",
        };
    }

    // Network (Purple)
    if name_lower.contains("pihole") || name_lower.contains("pi-hole") || name_lower.contains("adguard") ||
       name_lower.contains("nginx") || name_lower.contains("traefik") || name_lower.contains("caddy") || name_lower.contains("npm") ||
       name_lower.contains("unifi") || name_lower.contains("cloudflared") || name_lower.contains("cloudflare-ddns") ||
       name_lower.contains("ddclient") || name_lower.contains("duckdns") || name_lower.contains("swag") {
        let icon = if name_lower.contains("pihole") || name_lower.contains("pi-hole") || name_lower.contains("adguard") {
            "fa-shield"
        } else {
            "fa-exchange"
        };
        return StaticConfig {
            icon,
            color: "#9b59b6",
            bg: "rgba(155, 89, 182, 0.08)",
            border: "rgba(155, 89, 182, 0.2)",
        };
    }

    // VPN (Cool Emerald)
    if name_lower.contains("tailscale") || name_lower.contains("wireguard") || name_lower.contains("vpn") ||
       name_lower.contains("headscale") || name_lower.contains("netbird") || name_lower.contains("openvpn") ||
       name_lower.contains("pritunl") {
        return StaticConfig {
            icon: "fa-key",
            color: "#10ac84",
            bg: "rgba(16, 172, 132, 0.08)",
            border: "rgba(16, 172, 132, 0.2)",
        };
    }

    // Smart Home & IoT (Yellow)
    if name_lower.contains("home-assistant") || name_lower.contains("homeassistant") || name_lower.contains("hass") ||
       name_lower.contains("zigbee") || name_lower.contains("mqtt") || name_lower.contains("esphome") ||
       name_lower.contains("homebridge") || name_lower.contains("openhab") || name_lower.contains("jeedom") ||
       name_lower.contains("deconz") || name_lower.contains("wled") || name_lower.contains("scrypted") ||
       name_lower.contains("gladys") {
        let icon = if name_lower.contains("home-assistant") || name_lower.contains("homeassistant") || name_lower.contains("hass") {
            "fa-home"
        } else if name_lower.contains("homebridge") || name_lower.contains("openhab") || name_lower.contains("jeedom") {
            "fa-server"
        } else if name_lower.contains("wled") {
            "fa-lightbulb-o"
        } else {
            "fa-bolt"
        };
        return StaticConfig {
            icon,
            color: "#f1c40f",
            bg: "rgba(241, 196, 15, 0.08)",
            border: "rgba(241, 196, 15, 0.2)",
        };
    }

    // Security (Red)
    if name_lower.contains("vaultwarden") || name_lower.contains("bitwarden") || name_lower.contains("keepass") ||
       name_lower.contains("fail2ban") || name_lower.contains("crowdsec") || name_lower.contains("authentik") ||
       name_lower.contains("authelia") || name_lower.contains("keycloak") {
        return StaticConfig {
            icon: "fa-lock",
            color: "#e74c3c",
            bg: "rgba(231, 76, 60, 0.08)",
            border: "rgba(231, 76, 60, 0.2)",
        };
    }

    // Cloud (Blue)
    if name_lower.contains("nextcloud") || name_lower.contains("owncloud") ||
       name_lower.contains("seafile") || name_lower.contains("filerun") || name_lower.contains("immich") ||
       name_lower.contains("photoprism") || name_lower.contains("komga") || name_lower.contains("uboquity") {
        return StaticConfig {
            icon: "fa-cloud",
            color: "#74b9ff",
            bg: "rgba(116, 185, 255, 0.08)",
            border: "rgba(116, 185, 255, 0.2)",
        };
    }

    // Communication & Chat (Teal)
    if name_lower.contains("matrix-synapse") || name_lower.contains("mattermost") ||
       name_lower.contains("rocketchat") || name_lower.contains("mumble") ||
       name_lower.contains("teamspeak") || name_lower.contains("discourse") ||
       name_lower.contains("mailserver") || name_lower.contains("postfix") {
        let icon = if name_lower.contains("mail") || name_lower.contains("postfix") {
            "fa-envelope-o"
        } else {
            "fa-comments"
        };
        return StaticConfig {
            icon,
            color: "#00d2d3",
            bg: "rgba(0, 210, 211, 0.08)",
            border: "rgba(0, 210, 211, 0.2)",
        };
    }

    // Social Media (Pinkish Lilac)
    if name_lower.contains("mastodon") || name_lower.contains("wordpress") ||
       name_lower.contains("ghost") || name_lower.contains("linkding") ||
       name_lower.contains("linkwarden") || name_lower.contains("wallabag") ||
       name_lower.contains("shiori") || name_lower.contains("yourls") ||
       name_lower.contains("kutt") || name_lower.contains("humhub") ||
       name_lower.contains("friendica") {
        let icon = if name_lower.contains("wordpress") || name_lower.contains("ghost") {
            "fa-pencil-square-o"
        } else if name_lower.contains("linkding") || name_lower.contains("linkwarden") || name_lower.contains("shiori") {
            "fa-bookmark-o"
        } else {
            "fa-share-alt"
        };
        return StaticConfig {
            icon,
            color: "#ff9ff3",
            bg: "rgba(255, 159, 243, 0.08)",
            border: "rgba(255, 159, 243, 0.2)",
        };
    }

    // Backup (Teal)
    if name_lower.contains("duplicati") || name_lower.contains("duplicacy") || name_lower.contains("kopia") ||
       name_lower.contains("backups") || name_lower.contains("archivebox") || name_lower.contains("restic") ||
       name_lower.contains("borgbackup") || name_lower.contains("urbackup") {
        return StaticConfig {
            icon: "fa-cloud-upload",
            color: "#00d2d3",
            bg: "rgba(0, 210, 211, 0.08)",
            border: "rgba(0, 210, 211, 0.2)",
        };
    }

    // Sync (Mint Green)
    if name_lower.contains("syncthing") || name_lower.contains("rclone") || name_lower.contains("krusader") ||
       name_lower.contains("filezilla") || name_lower.contains("rsync") || name_lower.contains("resilio-sync") {
        let icon = if name_lower.contains("syncthing") {
            "fa-refresh"
        } else if name_lower.contains("krusader") || name_lower.contains("filezilla") {
            "fa-folder-open-o"
        } else {
            "fa-refresh"
        };
        return StaticConfig {
            icon,
            color: "#1abc9c",
            bg: "rgba(26, 188, 156, 0.08)",
            border: "rgba(26, 188, 156, 0.2)",
        };
    }

    // Databases & Monitoring (Grey-Blue)
    if name_lower.contains("influx") || name_lower.contains("prometheus") || name_lower.contains("grafana") ||
       name_lower.contains("kuma") || name_lower.contains("netdata") || name_lower.contains("postgres") ||
       name_lower.contains("mysql") || name_lower.contains("mariadb") || name_lower.contains("redis") ||
       name_lower.contains("mongo") {
        let icon = if name_lower.contains("influx") || name_lower.contains("postgres") || name_lower.contains("mysql") ||
                      name_lower.contains("mariadb") || name_lower.contains("redis") || name_lower.contains("mongo") {
            "fa-database"
        } else {
            "fa-bar-chart"
        };
        return StaticConfig {
            icon,
            color: "#6c7a89",
            bg: "rgba(108, 122, 137, 0.08)",
            border: "rgba(108, 122, 137, 0.2)",
        };
    }

    // Default Generic Server (Grey)
    StaticConfig {
        icon: "fa-server",
        color: "#7f8c8d",
        bg: "rgba(127, 140, 141, 0.08)",
        border: "rgba(127, 140, 141, 0.2)",
    }
}

fn get_service_fa_config(name: &str) -> FaIconConfig {
    let name_lower = name.to_lowercase();
    let static_cfg = get_static_config(&name_lower);
    
    let mut icon = static_cfg.icon.to_string();
    
    // Try to load preset JSON file to extract custom icon
    let preset_path = format!("/usr/local/emhttp/plugins/nix/presets/{}.json", name_lower);
    if std::path::Path::new(&preset_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&preset_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ic) = json.get("icon").and_then(|i| i.as_str()) {
                    icon = ic.to_string();
                }
            }
        }
    }
    
    FaIconConfig {
        icon,
        color: static_cfg.color,
        bg: static_cfg.bg,
        border: static_cfg.border,
    }
}

pub fn render_service_row(s: &ServiceStatus, config: &Option<ProcessComposeConfig>, host_ips: &[HostAddr]) -> String {
    let is_running = s.status.to_lowercase() == "running";
    let status_lower = s.status.to_lowercase();
    let is_stopped = status_lower == "stopped"
        || status_lower == "completed"
        || status_lower == "terminating";

    let status_subtext = if is_running {
        r#"<span style="color: #2ecc71;">●</span> started"#
    } else if is_stopped && s.exit_code.unwrap_or(0) == 0 {
        r#"<span style="color: #e74c3c;">●</span> stopped"#
    } else {
        r#"<span style="color: #f1c40f;">●</span> failed"#
    };

    let cmd = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .map(|p| p.command.as_str())
        .unwrap_or("");
    
    let uri = extract_package_uri(cmd).unwrap_or_else(|| format!("nixpkgs#{}", s.name));
    let version = get_cached_version(&uri);

    let version_badge = if version != "unknown" {
        if let Some(link_url) = get_package_link_url(&uri) {
            format!(
                r#"<div style="font-size: 11px; color: #a0a0a5; margin-top: -1px; margin-bottom: -1px;">v<a href="{}" target="_blank" style="color: #00a1ff; text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 8px;"></i></a> <span style="color: #2ecc71; font-weight: 500;">(up-to-date)</span></div>"#,
                link_url, version
            )
        } else {
            format!(
                r#"<div style="font-size: 11px; color: #a0a0a5; margin-top: -1px; margin-bottom: -1px;">v{} <span style="color: #2ecc71; font-weight: 500;">(up-to-date)</span></div>"#,
                version
            )
        }
    } else {
        "".to_string()
    };

    let cfg = get_service_fa_config(&s.name);
    let app_html = format!(
        r#"<div style="display: flex; align-items: center; gap: 10px;">
            <div style="width: 28px; height: 28px; border-radius: 4px; background: {}; border: 1px solid {}; display: inline-flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0;">
                <i class="fa {}" style="font-size: 14px;"></i>
            </div>
            <div style="display: flex; flex-direction: column; gap: 2px;">
                <strong style="font-size: 13px;">{}</strong>
                {}
                <div style="font-size: 11px; color: #a0a0a5;">{}</div>
            </div>
        </div>"#,
        cfg.bg, cfg.border, cfg.color, cfg.icon, s.name, version_badge, status_subtext
    );

    let port_num = get_service_web_port(&s.name);
    let metadata_file = format!("/boot/config/plugins/nix/metadata/{}.json", s.name);
    let mut bind_address_override = None;
    let mut extra_binds_vec = Vec::new();
    let mut gpus_override = None;
    let mut legacy_gpu = None;

    if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            bind_address_override = meta.get("bind_address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
                
            gpus_override = meta.get("gpus")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            legacy_gpu = meta.get("gpu")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
                
            if let Some(binds_val) = meta.get("extra_binds") {
                if let Some(binds_str) = binds_val.as_str() {
                    if let Ok(parsed_binds) = serde_json::from_str::<serde_json::Value>(binds_str) {
                        if let Some(arr) = parsed_binds.as_array() {
                            for item in arr {
                                if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|h| h.as_str()), item.get("sandbox").and_then(|s| s.as_str())) {
                                    extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                                }
                            }
                        }
                    }
                } else if let Some(arr) = binds_val.as_array() {
                    for item in arr {
                        if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|h| h.as_str()), item.get("sandbox").and_then(|s| s.as_str())) {
                            extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                        }
                    }
                }
            }
        }
    }

    let lan_ip_port_html = if let Some(port) = port_num {
        let mut ip_links = Vec::new();
        let has_specific_bind = if let Some(ref addr) = bind_address_override {
            let a = addr.trim();
            !a.is_empty() && a != "0.0.0.0" && a != "*"
        } else {
            false
        };

        for addr in host_ips {
            if has_specific_bind {
                if let Some(ref target) = bind_address_override {
                    if addr.ip != target.trim() {
                        continue;
                    }
                }
            }

            let label = match addr.interface.to_lowercase().as_str() {
                "tailscale0" | "tailscale" => "tailscale".to_string(),
                other => other.to_string(),
            };
            
            let link = if is_running {
                format!(
                    r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a> <span style="font-size: 10px; color: #777; font-family: monospace;">({})</span></div>"##,
                    addr.ip, port, addr.ip, port, label
                )
            } else {
                format!(
                    r##"<div style="margin-bottom: 4px;"><span style="color: #888;">{}:{}</span> <span style="font-size: 10px; color: #555; font-family: monospace;">({})</span></div>"##,
                    addr.ip, port, label
                )
            };
            ip_links.push(link);
        }

        if ip_links.is_empty() && has_specific_bind {
            if let Some(ref target) = bind_address_override {
                let link = if is_running {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><a href="#" onclick="window.open('http://{}:{}/', '_blank'); return false;" style="color: #00a1ff; text-decoration: none; font-weight: 500;">{}:{} <i class="fa fa-external-link" style="font-size: 9px; margin-left: 1px;"></i></a> <span style="font-size: 10px; color: #777; font-family: monospace;">(override)</span></div>"##,
                        target, port, target, port
                    )
                } else {
                    format!(
                        r##"<div style="margin-bottom: 4px;"><span style="color: #888;">{}:{}</span> <span style="font-size: 10px; color: #555; font-family: monospace;">(override)</span></div>"##,
                        target, port
                    )
                };
                ip_links.push(link);
            }
        }

        if ip_links.is_empty() {
            "-".to_string()
        } else {
            ip_links.join("")
        }
    } else {
        "-".to_string()
    };

    let home_path = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .map(|p| get_service_appdata_path(&s.name, &p.command))
        .unwrap_or_else(|| "-".to_string());

    let mut volume_mappings = Vec::new();
    if home_path != "-" && !home_path.is_empty() {
        volume_mappings.push(format!(
            r#"<div style="margin-bottom: 4px;"><span style="color: #a0a0a5; font-family: monospace;">/config</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code></div>"#,
            home_path
        ));
    }
    for (host, sandbox) in extra_binds_vec {
        if !host.is_empty() && !sandbox.is_empty() {
            volume_mappings.push(format!(
                r#"<div style="margin-bottom: 4px;"><span style="color: #a0a0a5; font-family: monospace;">{}</span> <i class="fa fa-arrow-right" style="margin: 0 4px; font-size: 10px; color: #888;"></i> <code>{}</code></div>"#,
                sandbox, host
            ));
        }
    }

    let volume_mappings_html = if volume_mappings.is_empty() {
        "-".to_string()
    } else {
        volume_mappings.join("")
    };

    let autostart_enabled = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .and_then(|p| p.availability.as_ref())
        .map(|a| a.restart.to_lowercase() == "always")
        .unwrap_or(true);

    let autostart_checked = if autostart_enabled { "checked" } else { "" };
    let autostart_html = format!(
        r#"<label class="nix-switch">
            <input type="checkbox" onchange="toggleAutostart('{}', this.checked)" {}>
            <span class="nix-slider"></span>
        </label>"#,
        s.name, autostart_checked
    );

    let start_btn = if !is_running {
        format!(r#"<button type="button" class="nix-btn" onclick="serviceAction('{}', 'start')" title="Start"><i class="fa fa-play"></i></button>"#, s.name)
    } else {
        format!(r#"<button type="button" class="nix-btn" disabled title="Service is running"><i class="fa fa-play"></i></button>"#)
    };

    let stop_btn = if is_running {
        format!(r#"<button type="button" class="nix-btn" onclick="serviceAction('{}', 'stop')" title="Stop"><i class="fa fa-stop"></i></button>"#, s.name)
    } else {
        format!(r#"<button type="button" class="nix-btn" disabled title="Service is stopped"><i class="fa fa-stop"></i></button>"#)
    };

    let edit_btn = format!(
        r#"<button type="button" class="nix-btn" onclick="editService('{}')" title="Edit Config"><i class="fa fa-edit"></i></button>"#,
        s.name
    );

    let logs_btn = format!(r#"<button type="button" class="nix-btn" onclick="openLogs('{}')" title="Logs"><i class="fa fa-file-text-o"></i></button>"#, s.name);

    let remove_btn = format!(
        r#"<button type="button" class="nix-btn" style="color: #e74c3c; border-color: #e74c3c;" onclick="removeService('{}')" title="Remove"><i class="fa fa-trash-o"></i></button>"#,
        s.name
    );

    let gpus_display = match gpus_override {
        Some(ref g) if !g.trim().is_empty() => {
            let mut badges = Vec::new();
            for part in g.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    let display_part = if p.starts_with("nvidia-") {
                        p.replace("nvidia-", "GPU-")
                    } else {
                        p.to_string()
                    };
                    badges.push(format!(
                        r#"<div style="margin-bottom: 4px;"><span style="background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.25); border-radius: 3px; padding: 2px 6px; font-size: 10px; color: #00a1ff; font-family: monospace; display: inline-block;">{}</span></div>"#,
                        display_part
                    ));
                }
            }
            if badges.is_empty() {
                r#"<span style="color: #777;">-</span>"#.to_string()
            } else {
                badges.join("")
            }
        }
        _ => {
            if let Some(ref lg) = legacy_gpu {
                if lg == "1" || lg == "true" {
                    r#"<div style="margin-bottom: 4px;"><span style="background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.25); border-radius: 3px; padding: 2px 6px; font-size: 10px; color: #00a1ff; font-family: monospace; display: inline-block;">All GPUs</span></div>"#.to_string()
                } else {
                    r#"<span style="color: #777;">-</span>"#.to_string()
                }
            } else {
                r#"<span style="color: #777;">-</span>"#.to_string()
            }
        }
    };

    let resources_html = {
        let mut res = String::new();
        if is_running {
            let cpu_str = if let Some(cpu) = s.cpu {
                format!("{:.1}% CPU", cpu)
            } else {
                "0.0% CPU".to_string()
            };
            let mem_str = if let Some(mem) = s.memory {
                let mb = mem as f64 / 1_048_576.0;
                format!("{:.1} MB RAM", mb)
            } else {
                "0.0 MB RAM".to_string()
            };
            res.push_str(&format!(
                r#"<div style="font-size: 11px; color: #eee; font-family: monospace; line-height: 1.4;">{}</div>
                   <div style="font-size: 11px; color: #a0a0a5; font-family: monospace; line-height: 1.4; margin-bottom: 4px;">{}</div>"#,
                cpu_str, mem_str
            ));
        }
        if gpus_display != r#"<span style="color: #777;">-</span>"# {
            res.push_str(&gpus_display);
        } else if !is_running {
            res.push_str(r#"<span style="color: #777;">-</span>"#);
        }
        res
    };

    format!(
        r#"<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <div class="nix-actions-wrapper">
                    {}
                    {}
                    {}
                    {}
                    {}
                </div>
            </td>
            <td>
                <div style="display: inline-block; vertical-align: middle;">{}</div>
            </td>
        </tr>"#,
        app_html, lan_ip_port_html, volume_mappings_html, resources_html, start_btn, stop_btn, edit_btn, logs_btn, remove_btn, autostart_html
    )
}
