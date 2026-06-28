function updatePresetInfo() {
    var uri = $("#custom-uri").val() || "";
    var name = uri.replace("nixpkgs#", "").toLowerCase().trim();
    var infoBox = $("#nix-preset-info-box");
    if (!infoBox.length) {
        $("#custom-uri").parent().after('<div id="nix-preset-info-box" style="margin-top: 10px; display: none;"></div>');
        infoBox = $("#nix-preset-info-box");
    }
    
    var presets = {
        "actual-budget": "Actual Budget", "adguard-home": "AdGuard Home", "agent-dvr": "Agent DVR", "airsonic-advanced": "Airsonic Advanced",
        "anythingllm": "AnythingLLM", "apprise": "Apprise", "appwrite": "Appwrite", "archivebox": "ArchiveBox",
        "audiobookshelf": "Audiobookshelf", "authelia": "Authelia", "authentik": "Authentik", "baserow": "Baserow",
        "bazarr": "Bazarr", "bitwarden": "Bitwarden", "bookstack": "BookStack", "borgbackup": "BorgBackup",
        "caddy": "Caddy", "calibre": "Calibre", "calibre-web": "Calibre-Web", "changedetection-io": "ChangeDetection.io",
        "clickhouse": "ClickHouse", "cloudbeaver": "CloudBeaver", "cloudflare-ddns": "Cloudflare DDNS", "cloudflared": "Cloudflare Tunnel",
        "code-server": "Code-server", "comfyui": "ComfyUI", "crowdsec": "CrowdSec", "cyberchef": "CyberChef",
        "dashy": "Dashy", "ddclient": "DDclient", "deconz": "deCONZ", "deluge": "Deluge",
        "dify": "Dify", "discourse": "Discourse", "dizquetv": "DizqueTV", "dokuwiki": "DokuWiki",
        "doplarr": "Doplarr", "dozzle": "Dozzle", "duckdns": "DuckDNS", "duplicacy": "Duplicacy",
        "duplicati": "Duplicati", "emby": "Emby", "emulatorjs": "EmulatorJS", "ersatztv": "ErsatzTV",
        "esphome": "ESPHome", "etherpad": "Etherpad", "excalidraw": "Excalidraw", "fail2ban": "Fail2ban",
        "filebot": "FileBot", "filerun": "Filerun", "filezilla": "FileZilla", "firefly-iii": "Firefly III",
        "flame": "Flame", "flaresolverr": "Flaresolverr", "flowise": "Flowise", "focalboard": "Focalboard",
        "fooocus": "Fooocus", "forgejo": "Forgejo", "friendica": "Friendica", "frigate": "Frigate NVR",
        "gatus": "Gatus", "ghost": "Ghost", "gitea": "Gitea", "gladys-assistant": "Gladys Assistant",
        "glances": "Glances", "glitchtip": "GlitchTip", "goaccess": "GoAccess", "gotify": "Gotify",
        "grafana": "Grafana", "grocy": "Grocy", "guacamole": "Apache Guacamole", "handbrake": "HandBrake",
        "headscale": "Headscale", "hedgedoc": "HedgeDoc", "heimdall": "Heimdall", "hoarder": "Hoarder",
        "homarr": "Homarr", "home-assistant": "Home Assistant", "homebridge": "Homebridge", "homepage": "Homepage",
        "humhub": "HumHub", "immich": "Immich", "influxdb": "InfluxDB", "invokeai": "InvokeAI",
        "it-tools": "IT-Tools", "jackett": "Jackett", "jdownloader2": "JDownloader 2", "jeedom": "Jeedom",
        "jellyfin": "Jellyfin", "jellyseerr": "Jellyseerr", "jellystat": "Jellystat", "joplin-server": "Joplin Server",
        "kanboard": "Kanboard", "kavita": "Kavita", "kerberos-io": "Kerberos.io", "keycloak": "Keycloak",
        "kometa": "Kometa", "komga": "Komga", "kopia": "Kopia", "krusader": "Krusader",
        "kutt": "Kutt", "lancache": "Lancache", "lazylibrarian": "LazyLibrarian", "leantime": "Leantime",
        "librechat": "LibreChat", "lidarr": "Lidarr", "linkding": "Linkding", "linkwarden": "Linkwarden",
        "lobe-chat": "Lobe Chat", "localai": "LocalAI", "mailserver": "Mail Server", "makemkv": "MakeMKV",
        "mariadb": "MariaDB", "mastodon": "Mastodon", "matomo": "Matomo", "matrix-synapse": "Matrix Synapse",
        "mattermost": "Mattermost", "mealie": "Mealie", "mediawiki": "MediaWiki", "minecraft-server": "Minecraft Server",
        "mosquitto": "Mosquitto", "mumble-server": "Mumble Server", "mylar3": "Mylar 3", "mysql": "MySQL",
        "n8n": "n8n", "navidrome": "Navidrome", "netbird": "Netbird", "netdata": "Netdata",
        "nextcloud": "Nextcloud", "nginx-proxy-manager": "Nginx Proxy Manager", "nocodb": "NocoDB", "node-red": "Node-RED",
        "ntfy": "NTFY", "nzbget": "NZBGet", "ollama": "Ollama", "ombi": "Ombi",
        "open-webui": "Open WebUI", "openhab": "OpenHAB", "openvpn": "OpenVPN", "organizr": "Organizr",
        "outline": "Outline", "overseerr": "Overseerr", "owncloud": "OwnCloud", "paperless-ngx": "Paperless-ngx",
        "petio": "Petio", "pgadmin4": "pgAdmin4", "photoprism": "PhotoPrism", "pi-hole": "Pi-hole",
        "plex": "Plex Media Server", "plex-anisync": "Plex-Anisync", "portainer": "Portainer", "postgresql": "PostgreSQL",
        "pritunl": "Pritunl", "prometheus": "Prometheus", "prowlarr": "Prowlarr", "pterodactyl": "Pterodactyl",
        "pufferpanel": "PufferPanel", "pyload": "PyLoad", "pymedusa": "PyMedusa", "qbittorrent": "qBittorrent",
        "qbittorrent-vpn": "qBittorrent VPN", "radarr": "Radarr", "rclone": "Rclone", "readarr": "Readarr",
        "recyclarr": "Recyclarr", "redis": "Redis", "requestrr": "Requestrr", "resilio-sync": "Resilio Sync",
        "restic": "Restic", "rocketchat": "Rocket.Chat", "romm": "RomM", "rsync": "Rsync",
        "sabnzbd": "SABnzbd", "scrutiny": "Scrutiny", "scrypted": "Scrypted", "seafile": "Seafile",
        "sentry": "Sentry", "shinobi": "Shinobi", "shiori": "Shiori", "shlink": "Shlink",
        "shoko-server": "Shoko Server", "sickchill": "SickChill", "singlefile": "SingleFile", "sonarr": "Sonarr",
        "speedtest-tracker": "Speedtest Tracker", "sqlite-web": "SQLite-Web", "stable-diffusion-webui": "Stable Diffusion WebUI", "statping-ng": "Statping-ng",
        "steamcmd": "SteamCMD", "stirling-pdf": "Stirling-PDF", "supabase": "Supabase", "swag": "SWAG",
        "syncthing": "Syncthing", "tailscale": "Tailscale", "tasmoadmin": "TasmoAdmin", "tautulli": "Tautulli",
        "tdarr": "Tdarr", "teamspeak-server": "TeamSpeak Server", "telegraf": "Telegraf", "text-generation-webui": "Text Generation WebUI",
        "threadfin": "Threadfin", "traefik": "Traefik", "transmission": "Transmission", "transmission-openvpn": "Transmission VPN",
        "trilium-notes": "Trilium Notes", "tubearchivist": "Tube Archivist", "uboquity": "Uboquity", "unifi": "UniFi Network Application",
        "unmanic": "Unmanic", "uptime-kuma": "Uptime Kuma", "vaultwarden": "Vaultwarden", "vikunja": "Vikunja",
        "wallabag": "Wallabag", "wekan": "Wekan", "whisparr": "Whisparr", "wiki-js": "Wiki.js",
        "wireguard": "WireGuard", "wireguard-ui": "Wireguard-UI", "wled": "WLED", "wordpress": "WordPress",
        "xteve": "xTeVe", "yourls": "YOURLS", "zigbee2mqtt": "Zigbee2MQTT", "zoneminder": "ZoneMinder",
        "zwavejs2mqtt": "ZwaveJS2MQTT"
    };

    var icons = {
        "actual-budget": "fa-server", "adguard-home": "fa-shield", "agent-dvr": "fa-server", "airsonic-advanced": "fa-music",
        "anythingllm": "fa-magic", "apprise": "fa-server", "appwrite": "fa-server", "archivebox": "fa-cloud-upload",
        "audiobookshelf": "fa-book", "authelia": "fa-lock", "authentik": "fa-lock", "baserow": "fa-server",
        "bazarr": "fa-closed-captioning", "bitwarden": "fa-lock", "bookstack": "fa-file-text-o", "borgbackup": "fa-cloud-upload",
        "caddy": "fa-exchange", "calibre": "fa-book", "calibre-web": "fa-book", "changedetection-io": "fa-server",
        "clickhouse": "fa-database", "cloudbeaver": "fa-server", "cloudflare-ddns": "fa-server", "cloudflared": "fa-server",
        "code-server": "fa-server", "comfyui": "fa-magic", "crowdsec": "fa-lock", "cyberchef": "fa-server",
        "dashy": "fa-tachometer", "ddclient": "fa-server", "deconz": "fa-bolt", "deluge": "fa-download",
        "dify": "fa-server", "discourse": "fa-comments", "dizquetv": "fa-server", "dokuwiki": "fa-file-text-o",
        "doplarr": "fa-server", "dozzle": "fa-server", "duckdns": "fa-server", "duplicacy": "fa-cloud-upload",
        "duplicati": "fa-cloud-upload", "emby": "fa-play-circle", "emulatorjs": "fa-gamepad", "ersatztv": "fa-server",
        "esphome": "fa-bolt", "etherpad": "fa-file-text-o", "excalidraw": "fa-file-text-o", "fail2ban": "fa-lock",
        "filebot": "fa-server", "filerun": "fa-cloud", "filezilla": "fa-folder-open-o", "firefly-iii": "fa-server",
        "flame": "fa-tachometer", "flaresolverr": "fa-server", "flowise": "fa-magic", "focalboard": "fa-file-text-o",
        "fooocus": "fa-server", "forgejo": "fa-server", "friendica": "fa-share-alt", "frigate": "fa-server",
        "gatus": "fa-server", "ghost": "fa-pencil-square-o", "gitea": "fa-server", "gladys-assistant": "fa-home",
        "glances": "fa-bar-chart", "glitchtip": "fa-server", "goaccess": "fa-server", "gotify": "fa-server",
        "grafana": "fa-bar-chart", "grocy": "fa-server", "guacamole": "fa-server", "handbrake": "fa-server",
        "headscale": "fa-key", "hedgedoc": "fa-file-text-o", "heimdall": "fa-tachometer", "hoarder": "fa-server",
        "homarr": "fa-tachometer", "home-assistant": "fa-home", "homebridge": "fa-home", "homepage": "fa-tachometer",
        "humhub": "fa-share-alt", "immich": "fa-cloud", "influxdb": "fa-database", "invokeai": "fa-server",
        "it-tools": "fa-server", "jackett": "fa-search", "jdownloader2": "fa-server", "jeedom": "fa-home",
        "jellyfin": "fa-play-circle", "jellyseerr": "fa-server", "jellystat": "fa-server", "joplin-server": "fa-file-text-o",
        "kanboard": "fa-file-text-o", "kavita": "fa-server", "kerberos-io": "fa-server", "keycloak": "fa-lock",
        "kometa": "fa-server", "komga": "fa-cloud", "kopia": "fa-cloud-upload", "krusader": "fa-folder-open-o",
        "kutt": "fa-share-alt", "lancache": "fa-gamepad", "lazylibrarian": "fa-server", "leantime": "fa-file-text-o",
        "librechat": "fa-magic", "lidarr": "fa-music", "linkding": "fa-bookmark-o", "linkwarden": "fa-bookmark-o",
        "lobe-chat": "fa-server", "localai": "fa-magic", "mailserver": "fa-envelope-o", "makemkv": "fa-server",
        "mariadb": "fa-database", "mastodon": "fa-share-alt", "matomo": "fa-server", "matrix-synapse": "fa-comments",
        "mattermost": "fa-comments", "mealie": "fa-server", "mediawiki": "fa-file-text-o", "minecraft-server": "fa-gamepad",
        "mosquitto": "fa-server", "mumble-server": "fa-comments", "mylar3": "fa-server", "mysql": "fa-database",
        "n8n": "fa-server", "navidrome": "fa-music", "netbird": "fa-key", "netdata": "fa-bar-chart",
        "nextcloud": "fa-cloud", "nginx-proxy-manager": "fa-exchange", "nocodb": "fa-server", "node-red": "fa-code-fork",
        "ntfy": "fa-server", "nzbget": "fa-download", "ollama": "fa-magic", "ombi": "fa-server",
        "open-webui": "fa-magic", "openhab": "fa-home", "openvpn": "fa-key", "organizr": "fa-tachometer",
        "outline": "fa-file-text-o", "overseerr": "fa-server", "owncloud": "fa-cloud", "paperless-ngx": "fa-file-text-o",
        "petio": "fa-server", "pgadmin4": "fa-server", "photoprism": "fa-cloud", "pi-hole": "fa-shield",
        "plex": "fa-play-circle", "plex-anisync": "fa-play-circle", "portainer": "fa-server", "postgresql": "fa-database",
        "pritunl": "fa-key", "prometheus": "fa-bar-chart", "prowlarr": "fa-search", "pterodactyl": "fa-gamepad",
        "pufferpanel": "fa-gamepad", "pyload": "fa-server", "pymedusa": "fa-server", "qbittorrent": "fa-download",
        "qbittorrent-vpn": "fa-download", "radarr": "fa-film", "rclone": "fa-refresh", "readarr": "fa-book",
        "recyclarr": "fa-server", "redis": "fa-database", "requestrr": "fa-server", "resilio-sync": "fa-refresh",
        "restic": "fa-cloud-upload", "rocketchat": "fa-comments", "romm": "fa-gamepad", "rsync": "fa-refresh",
        "sabnzbd": "fa-download", "scrutiny": "fa-server", "scrypted": "fa-bolt", "seafile": "fa-cloud",
        "sentry": "fa-server", "shinobi": "fa-server", "shiori": "fa-bookmark-o", "shlink": "fa-server",
        "shoko-server": "fa-server", "sickchill": "fa-television", "singlefile": "fa-server", "sonarr": "fa-television",
        "speedtest-tracker": "fa-server", "sqlite-web": "fa-database", "stable-diffusion-webui": "fa-magic", "statping-ng": "fa-server",
        "steamcmd": "fa-gamepad", "stirling-pdf": "fa-file-text-o", "supabase": "fa-server", "swag": "fa-exchange",
        "syncthing": "fa-refresh", "tailscale": "fa-key", "tasmoadmin": "fa-server", "tautulli": "fa-server",
        "tdarr": "fa-server", "teamspeak-server": "fa-comments", "telegraf": "fa-server", "text-generation-webui": "fa-magic",
        "threadfin": "fa-server", "traefik": "fa-exchange", "transmission": "fa-download", "transmission-openvpn": "fa-download",
        "trilium-notes": "fa-file-text-o", "tubearchivist": "fa-server", "uboquity": "fa-cloud", "unifi": "fa-server",
        "unmanic": "fa-server", "uptime-kuma": "fa-bar-chart", "vaultwarden": "fa-lock", "vikunja": "fa-server",
        "wallabag": "fa-share-alt", "wekan": "fa-file-text-o", "whisparr": "fa-server", "wiki-js": "fa-file-text-o",
        "wireguard": "fa-key", "wireguard-ui": "fa-key", "wled": "fa-lightbulb-o", "wordpress": "fa-pencil-square-o",
        "xteve": "fa-server", "yourls": "fa-share-alt", "zigbee2mqtt": "fa-bolt", "zoneminder": "fa-server",
        "zwavejs2mqtt": "fa-bolt"
    };

    var matched = presets[name];
    if (matched) {
        var iconClass = (typeof icons !== 'undefined' && icons[name]) ? icons[name] : "fa-info-circle";
        infoBox.html(`<div style="background: var(--nix-bg-secondary); border: 1px solid var(--nix-accent); border-radius: 4px; padding: 12px; font-size: 12px; color: var(--nix-text-primary); margin-bottom: 15px;">` +
            `<div style="font-weight: 600; color: #00a1ff; margin-bottom: 8px; display: flex; align-items: center; gap: 8px;">` +
            `<div style="width: 22px; height: 22px; border-radius: 3px; background: rgba(0, 161, 255, 0.08); border: 1px solid rgba(0, 161, 255, 0.2); display: inline-flex; align-items: center; justify-content: center; color: #00a1ff; flex-shrink: 0;"><i class="fa ${iconClass}" style="font-size: 11px;"></i></div>` +
            `Service Preset Detected: ${matched}</div>` +
            `<ul style="margin: 0; padding-left: 15px; line-height: 1.6;">` +
            `<li><strong>Service Name:</strong> <code>${name}</code></li>` +
            `<li><strong>Flake URI:</strong> <code>${uri}</code></li>` +
            `</ul></div>`).slideDown();
        if (window.currentPreset !== name) {
            window.currentPreset = name;
            if (!$("#custom-uri").prop('readonly')) $("#nix-ports-container").empty();
        }
    } else {
        infoBox.slideUp().empty();
        if (window.currentPreset !== '') {
            window.currentPreset = '';
            if (!$("#custom-uri").prop('readonly')) $("#nix-ports-container").empty();
        }
    }
}

function addEnvVarRow(key, value) {
    key = key || '';
    value = value || '';
    var row = $('<div class="nix-env-row" style="display: flex; gap: 10px; align-items: center; margin-bottom: 6px;">' +
        '<input type="text" class="nix-env-key" placeholder="VARIABLE_NAME" value="' + key + '" style="flex: 1; background: var(--nix-bg-secondary); border: 1px solid var(--nix-border-primary); border-radius: 4px; padding: 6px; color: var(--nix-text-primary);" autocomplete="off" spellcheck="false">' +
        '<span style="color: var(--nix-text-muted);">=</span>' +
        '<input type="text" class="nix-env-val" placeholder="Value" value="' + value + '" style="flex: 1.5; background: var(--nix-bg-secondary); border: 1px solid var(--nix-border-primary); border-radius: 4px; padding: 6px; color: var(--nix-text-primary);" autocomplete="off" spellcheck="false">' +
        '<button type="button" class="nix-btn" style="margin: 0; padding: 6px 10px; color: #e74c3c; border-color: #e74c3c; background: transparent; cursor: pointer;" onclick="$(this).parent().remove()"><i class="fa fa-trash"></i></button>' +
        '</div>');
    $("#nix-env-vars-container").append(row);
}
