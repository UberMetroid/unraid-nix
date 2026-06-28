use super::StaticConfig;

pub fn match_static_config(name_lower: &str) -> Option<StaticConfig> {
    // AI & LLMs (Indigo/Cyberpunk Purple-Blue)
    if name_lower.contains("ollama")
        || name_lower.contains("open-webui")
        || name_lower.contains("localai")
        || name_lower.contains("anythingllm")
        || name_lower.contains("librechat")
        || name_lower.contains("flowise")
        || name_lower.contains("stable-diffusion")
        || name_lower.contains("comfyui")
        || name_lower.contains("text-generation-webui")
        || name_lower.contains("invokeai")
        || name_lower.contains("fooocus")
        || name_lower.contains("dify")
        || name_lower.contains("lobe-chat")
    {
        return Some(StaticConfig {
            icon: "fa-magic",
            color: "#a29bfe",
            bg: "rgba(162, 155, 254, 0.08)",
            border: "rgba(162, 155, 254, 0.2)",
        });
    }

    // Dashboards (Amber/Gold)
    if name_lower.contains("homepage")
        || name_lower.contains("homarr")
        || name_lower.contains("heimdall")
        || name_lower.contains("dashy")
        || name_lower.contains("flame")
        || name_lower.contains("organizr")
    {
        return Some(StaticConfig {
            icon: "fa-tachometer",
            color: "#fdcb6e",
            bg: "rgba(253, 203, 110, 0.08)",
            border: "rgba(253, 203, 110, 0.2)",
        });
    }

    // Gaming (Coral/Red)
    if name_lower.contains("emulatorjs")
        || name_lower.contains("romm")
        || name_lower.contains("pterodactyl")
        || name_lower.contains("pufferpanel")
        || name_lower.contains("lancache")
        || name_lower.contains("minecraft-server")
        || name_lower.contains("steamcmd")
    {
        return Some(StaticConfig {
            icon: "fa-gamepad",
            color: "#ff7675",
            bg: "rgba(255, 118, 117, 0.08)",
            border: "rgba(255, 118, 117, 0.2)",
        });
    }

    // Productivity (Emerald Green)
    if name_lower.contains("paperless-ngx")
        || name_lower.contains("stirling-pdf")
        || name_lower.contains("joplin-server")
        || name_lower.contains("trilium-notes")
        || name_lower.contains("bookstack")
        || name_lower.contains("wiki-js")
        || name_lower.contains("dokuwiki")
        || name_lower.contains("mediawiki")
        || name_lower.contains("wekan")
        || name_lower.contains("focalboard")
        || name_lower.contains("leantime")
        || name_lower.contains("kanboard")
        || name_lower.contains("etherpad")
        || name_lower.contains("hedgedoc")
        || name_lower.contains("outline")
        || name_lower.contains("excalidraw")
    {
        return Some(StaticConfig {
            icon: "fa-file-text-o",
            color: "#2ecc71",
            bg: "rgba(46, 204, 113, 0.08)",
            border: "rgba(46, 204, 113, 0.2)",
        });
    }

    // Media Players (Cyan/Blue)
    if name_lower.contains("jellyfin")
        || name_lower.contains("plex")
        || name_lower.contains("emby")
        || name_lower.contains("navidrome")
        || name_lower.contains("airsonic")
        || name_lower.contains("subsonic")
        || name_lower.contains("lidarr")
        || name_lower.contains("audiobookshelf")
        || name_lower.contains("kavita")
        || name_lower.contains("calibre")
    {
        let icon = if name_lower.contains("jellyfin")
            || name_lower.contains("plex")
            || name_lower.contains("emby")
        {
            "fa-play-circle"
        } else if name_lower.contains("navidrome")
            || name_lower.contains("airsonic")
            || name_lower.contains("subsonic")
            || name_lower.contains("lidarr")
        {
            "fa-music"
        } else if name_lower.contains("calibre")
            || name_lower.contains("audiobookshelf")
            || name_lower.contains("kavita")
        {
            "fa-book"
        } else {
            "fa-music"
        };
        return Some(StaticConfig {
            icon,
            color: "#00a1ff",
            bg: "rgba(0, 161, 255, 0.08)",
            border: "rgba(0, 161, 255, 0.2)",
        });
    }

    // Servarr (Orange)
    if name_lower.contains("sonarr")
        || name_lower.contains("sickrage")
        || name_lower.contains("sickchill")
        || name_lower.contains("radarr")
        || name_lower.contains("couchpotato")
        || name_lower.contains("readarr")
        || name_lower.contains("bazarr")
        || name_lower.contains("prowlarr")
        || name_lower.contains("jackett")
    {
        let icon = if name_lower.contains("sonarr")
            || name_lower.contains("sickrage")
            || name_lower.contains("sickchill")
        {
            "fa-television"
        } else if name_lower.contains("radarr") || name_lower.contains("couchpotato") {
            "fa-film"
        } else if name_lower.contains("readarr") {
            "fa-book"
        } else if name_lower.contains("bazarr") {
            "fa-closed-captioning"
        } else if name_lower.contains("prowlarr") || name_lower.contains("jackett") {
            "fa-search"
        } else {
            "fa-television"
        };
        return Some(StaticConfig {
            icon,
            color: "#e67e22",
            bg: "rgba(230, 126, 34, 0.08)",
            border: "rgba(230, 126, 34, 0.2)",
        });
    }

    // General Automation & Workflows (Pink/Magenta)
    if name_lower.contains("n8n")
        || name_lower.contains("node-red")
        || name_lower.contains("nodered")
        || name_lower.contains("changedetection")
        || name_lower.contains("apprise")
        || name_lower.contains("gotify")
        || name_lower.contains("ntfy")
        || name_lower.contains("huginn")
        || name_lower.contains("activepieces")
    {
        return Some(StaticConfig {
            icon: "fa-code-fork",
            color: "#e84393",
            bg: "rgba(232, 67, 147, 0.08)",
            border: "rgba(232, 67, 147, 0.2)",
        });
    }

    // Download Clients (Green)
    if name_lower.contains("transmission")
        || name_lower.contains("sabnzbd")
        || name_lower.contains("nzbget")
        || name_lower.contains("qbittorrent")
        || name_lower.contains("deluge")
        || name_lower.contains("rtorrent")
        || name_lower.contains("aria2")
    {
        return Some(StaticConfig {
            icon: "fa-download",
            color: "#2ecc71",
            bg: "rgba(46, 204, 113, 0.08)",
            border: "rgba(46, 204, 113, 0.2)",
        });
    }

    // Network (Purple)
    if name_lower.contains("pihole")
        || name_lower.contains("pi-hole")
        || name_lower.contains("adguard")
        || name_lower.contains("nginx")
        || name_lower.contains("traefik")
        || name_lower.contains("caddy")
        || name_lower.contains("npm")
        || name_lower.contains("unifi")
        || name_lower.contains("cloudflared")
        || name_lower.contains("cloudflare-ddns")
        || name_lower.contains("ddclient")
        || name_lower.contains("duckdns")
        || name_lower.contains("swag")
    {
        let icon = if name_lower.contains("pihole")
            || name_lower.contains("pi-hole")
            || name_lower.contains("adguard")
        {
            "fa-shield"
        } else {
            "fa-exchange"
        };
        return Some(StaticConfig {
            icon,
            color: "#9b59b6",
            bg: "rgba(155, 89, 182, 0.08)",
            border: "rgba(155, 89, 182, 0.2)",
        });
    }

    // VPN (Cool Emerald)
    if name_lower.contains("tailscale")
        || name_lower.contains("wireguard")
        || name_lower.contains("vpn")
        || name_lower.contains("headscale")
        || name_lower.contains("netbird")
        || name_lower.contains("openvpn")
        || name_lower.contains("pritunl")
    {
        return Some(StaticConfig {
            icon: "fa-key",
            color: "#10ac84",
            bg: "rgba(16, 172, 132, 0.08)",
            border: "rgba(16, 172, 132, 0.2)",
        });
    }

    // Smart Home & IoT (Yellow)
    if name_lower.contains("home-assistant")
        || name_lower.contains("homeassistant")
        || name_lower.contains("hass")
        || name_lower.contains("zigbee")
        || name_lower.contains("mqtt")
        || name_lower.contains("esphome")
        || name_lower.contains("homebridge")
        || name_lower.contains("openhab")
        || name_lower.contains("jeedom")
        || name_lower.contains("deconz")
        || name_lower.contains("wled")
        || name_lower.contains("scrypted")
        || name_lower.contains("gladys")
    {
        let icon = if name_lower.contains("home-assistant")
            || name_lower.contains("homeassistant")
            || name_lower.contains("hass")
        {
            "fa-home"
        } else if name_lower.contains("homebridge")
            || name_lower.contains("openhab")
            || name_lower.contains("jeedom")
        {
            "fa-server"
        } else if name_lower.contains("wled") {
            "fa-lightbulb-o"
        } else {
            "fa-bolt"
        };
        return Some(StaticConfig {
            icon,
            color: "#f1c40f",
            bg: "rgba(241, 196, 15, 0.08)",
            border: "rgba(241, 196, 15, 0.2)",
        });
    }

    // Security (Red)
    if name_lower.contains("vaultwarden")
        || name_lower.contains("bitwarden")
        || name_lower.contains("keepass")
        || name_lower.contains("fail2ban")
        || name_lower.contains("crowdsec")
        || name_lower.contains("authentik")
        || name_lower.contains("authelia")
        || name_lower.contains("keycloak")
    {
        return Some(StaticConfig {
            icon: "fa-lock",
            color: "#e74c3c",
            bg: "rgba(231, 76, 60, 0.08)",
            border: "rgba(231, 76, 60, 0.2)",
        });
    }

    None
}
