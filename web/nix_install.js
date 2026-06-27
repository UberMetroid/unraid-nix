window.initInstallForm = function() {
    $("#custom-uri").val("").prop('readonly', false);
    $("#custom-appdata").val("");
    $("#custom-puid").val("99");
    $("#custom-pgid").val("100");
    $("#nix-ports-container").empty();
    $("#custom-bind-address").val("0.0.0.0");
    $("#nix-extra-binds-container").empty();
    $("#nix-install-section h3").text("Configure Flake");
    $("#nix-install-section .nix-subtext").text("Run or daemonize any custom flake from GitHub or a local directory.");
    $("#nix-install-submit-btn").text("Install Flake");

    window.legacyGpuEnabled = false;
    var selectedGpus = '';

    var editDataStr = sessionStorage.getItem('nix_edit_metadata');
    if (editDataStr) {
        sessionStorage.removeItem('nix_edit_metadata');
        var editData = JSON.parse(editDataStr);
        $("#custom-uri").val(editData.uri).prop('readonly', true);
        $("#custom-appdata").val(editData.appdata);
        $("#custom-puid").val(editData.puid);
        $("#custom-pgid").val(editData.pgid);
        
        if (editData.gpu === '1' || editData.gpu === 'true') {
            window.legacyGpuEnabled = true;
        }
        if (editData.gpus) {
            selectedGpus = editData.gpus;
        }
        
        var presetName = editData.uri.replace("nixpkgs#", "").toLowerCase().trim();
        populatePortRows(editData.port || '', presetName);

        if (editData.bind_address) {
            var select = $("#custom-bind-address");
            if (select.find("option[value='" + editData.bind_address + "']").length === 0) {
                select.append($("<option>", { value: editData.bind_address, text: editData.bind_address + " (Custom)" }));
            }
            select.val(editData.bind_address);
        }
        if (editData.extra_binds) {
            try {
                var binds = typeof editData.extra_binds === 'string' ? JSON.parse(editData.extra_binds) : editData.extra_binds;
                if (Array.isArray(binds)) { binds.forEach(function(b) { addBindRow(b.host, b.sandbox); }); }
            } catch(e) {}
        }
        $("#nix-install-section h3").text("Configure Flake: " + editData.name);
        $("#nix-install-section .nix-subtext").text("Modify paths, environment settings, and permissions for service: " + editData.name);
        $("#nix-install-submit-btn").text("Apply Changes");
    } else {
        var uri = sessionStorage.getItem('nix_install_uri');
        if (uri) {
            sessionStorage.removeItem('nix_install_uri');
            $("#custom-uri").val(uri);
            $("#custom-type").val('service');
            toggleServiceOptions();
            var name = uri.replace('nixpkgs#', '').split('/').pop().split(':').pop().split('#').pop().replace(/[^a-zA-Z0-9_-]/g, '');
            if (name) {
                var appdataRoot = window.NIX_APPDATA_ROOT;
                if (appdataRoot) { $("#custom-appdata").val(appdataRoot + "/" + name); }
                populatePortRows('', name);
            }
        }
    }
    loadAndRenderGpus(selectedGpus);
    updatePresetInfo();
};

function loadAndRenderGpus(selectedGpus) {
    var container = $("#nix-gpus-list");
    container.html('<div style="font-size: 11px; color: #888; font-style: italic;" id="nix-gpus-loading">Scanning host for GPU devices...</div>');
    
    $.getJSON('/plugins/nix/api.php?action=detect-gpus', function(gpus) {
        container.empty();
        if (!gpus || gpus.length === 0) {
            container.html('<div style="font-size: 12px; color: #888;">No GPU devices detected on host.</div>');
            return;
        }
        
        var selectedList = selectedGpus ? selectedGpus.split(',') : [];
        
        gpus.forEach(function(gpu) {
            var isChecked = selectedList.indexOf(gpu.id) !== -1;
            // Handle backward compatibility where legacy GPU is enabled but no custom select list
            if (!selectedGpus && window.legacyGpuEnabled) {
                isChecked = true;
            }
            
            var checkboxHtml = `<label style="display: flex; align-items: center; gap: 8px; font-weight: normal; margin: 0; cursor: pointer;">` +
                `<input type="checkbox" class="nix-gpu-checkbox" value="${gpu.id}" ${isChecked ? 'checked' : ''}>` +
                `<span style="color: #eee;">${gpu.name}</span>` +
                `</label>`;
            container.append(checkboxHtml);
        });
    }).fail(function() {
        container.html('<div style="font-size: 12px; color: #e74c3c;">Failed to scan host GPU devices.</div>');
    });
}

$(function() {
    if (typeof $.fn.fileTreeAttach === 'function') { $("#custom-appdata").fileTreeAttach(); }
    $(document).on('click', '.nix-folder-picker-btn', function() {
        $(this).siblings('input').focus().trigger('click');
    });
    window.initInstallForm();
});

function toggleServiceOptions() {
    var type = $("#custom-type").val();
    if (type === 'service') { $("#nix-service-only-options").slideDown(); }
    else { $("#nix-service-only-options").slideUp(); }
}

function installCustomFlake(e) {
    e.preventDefault();
    var submitBtn = $("#nix-install-submit-btn");
    if (submitBtn.prop('disabled')) return;
    
    var uri = $("#custom-uri").val();
    var type = $("#custom-type").val();
    
    if (type === 'service' && !$("#custom-appdata").val()) {
        alert("Configuration Location is required for services.");
        return;
    }
    
    var selectedGpus = $(".nix-gpu-checkbox:checked").map(function() {
        return this.value;
    }).get().join(',');
    var gpuVal = selectedGpus ? '1' : '0';

    var width = 900;
    var height = 600;
    var left = (window.screen.width - width) / 2;
    var top = (window.screen.height - height) / 2;
    var popupName = 'nix_install_popup_' + Date.now();
    var popup = window.open('', popupName, `scrollbars=yes,resizable=yes,status=no,location=no,toolbar=no,menubar=no,width=${width},height=${height},left=${left},top=${top}`);
    
    if (!popup) {
        alert("Popup blocker prevented opening the installation console. Please allow popups for this site.");
        return;
    }
    
    var form = $('<form>', { method: 'POST', action: '/plugins/nix/stream.php', target: popupName });
    var params = { csrf_token: window.csrf_token || '', action: 'install-custom', uri: uri, type: type };
    if (type === 'service') {
        Object.assign(params, {
            appdata: $("#custom-appdata").val(), media: '', puid: $("#custom-puid").val(), pgid: $("#custom-pgid").val(),
            gpu: gpuVal, gpus: selectedGpus, bind_address: $("#custom-bind-address").val()
        });
        var ports = $(".nix-port-row").map(function() {
            var host = $(this).find(".nix-port-host").val();
            var container = $(this).find(".nix-port-container").val();
            return (host && container) ? host + ":" + container : null;
        }).get().join(',');
        params.port = ports;

        var extraBinds = $(".nix-bind-row").map(function() {
            var host = $(this).find(".nix-bind-host").val();
            var sandbox = $(this).find(".nix-bind-sandbox").val();
            return (host && sandbox) ? { host: host, sandbox: sandbox } : null;
        }).get();
        params.extra_binds = JSON.stringify(extraBinds);
    }
    $.each(params, (k, v) => form.append($('<input>', { type: 'hidden', name: k, value: v })));
    
    $('body').append(form);
    form.submit();
    form.remove();
    
    var timer = setInterval(function() {
        if (popup.closed) {
            clearInterval(timer);
            if (type === 'service') { window.location.href = '/Nix/nix_services'; }
        }
    }, 1000);
}

var bindRowIndex = 0;
function addBindRow(hostVal, sandboxVal) {
    hostVal = hostVal || '';
    sandboxVal = sandboxVal || '';
    var idx = bindRowIndex++;
    var html = `<div class="nix-form-row nix-bind-row" id="bind-row-${idx}" style="margin-bottom: 8px; display: flex; gap: 8px; align-items: center;">` +
        `<div style="flex: 2; position: relative;">` +
        `<input type="text" class="nix-bind-host" id="bind-host-${idx}" value="${hostVal}" placeholder="Host Path (e.g. /mnt/user/downloads)" autocomplete="off" spellcheck="false" data-pickcloseonfile="true" data-pickfilter="HIDE_FILES_FILTER" data-pickroot="/mnt" data-pickfolders="true" style="padding-right: 30px;">` +
        `<i class="fa fa-folder-open nix-folder-picker-btn" style="position: absolute; right: 10px; top: 50%; transform: translateY(-50%); color: #aaa; cursor: pointer;"></i>` +
        `</div><div style="flex: 1;"><input type="text" class="nix-bind-sandbox" placeholder="Sandbox Path (e.g. /downloads)" value="${sandboxVal}"></div>` +
        `<div><button type="button" class="nix-btn" style="color: #e74c3c; border-color: #e74c3c; margin: 0; padding: 8px 12px;" onclick="removeBindRow(${idx})"><i class="fa fa-times"></i></button></div></div>`;
    $("#nix-extra-binds-container").append(html);
    if (typeof $.fn.fileTreeAttach === 'function') { $(`#bind-host-${idx}`).fileTreeAttach(); }
}
function removeBindRow(idx) { $(`#bind-row-${idx}`).remove(); }

var portRowIndex = 0;
fnPortText = (v) => v ? `<span style="font-size: 11px; color: #888; margin-left: 5px;">(${v})</span>` : '';
function addPortRow(hostVal, containerVal, labelVal, isPresetPort) {
    hostVal = hostVal || '';
    containerVal = containerVal || '';
    labelVal = labelVal || '';
    isPresetPort = isPresetPort || false;
    var idx = portRowIndex++;
    var readonlyContainer = isPresetPort ? 'readonly style="background: rgba(255,255,255,0.05); color: #888;"' : '';
    var deleteBtn = `<button type="button" class="nix-btn" style="color: #e74c3c; border-color: #e74c3c; margin: 0; padding: 8px 12px;" onclick="removePortRow(${idx})"><i class="fa fa-times"></i></button>`;
    var html = `<div class="nix-form-row nix-port-row" id="port-row-${idx}" style="margin-bottom: 8px; display: flex; gap: 8px; align-items: center;">` +
        `<div style="flex: 1;"><label style="font-size: 11px; margin-bottom: 4px; display: block;">Host Port</label>` +
        `<input type="number" class="nix-port-host" id="port-host-${idx}" value="${hostVal}" placeholder="e.g. 8096" min="1" max="65535" required></div>` +
        `<div style="display: flex; align-items: center; justify-content: center; padding-top: 15px;"><i class="fa fa-arrow-right" style="color: #888;"></i></div>` +
        `<div style="flex: 1;"><label style="font-size: 11px; margin-bottom: 4px; display: block;">Container Port ${fnPortText(labelVal)}</label>` +
        `<input type="number" class="nix-port-container" id="port-container-${idx}" value="${containerVal}" placeholder="e.g. 8096" min="1" max="65535" ${readonlyContainer} required></div>` +
        `<div style="padding-top: 15px;">${deleteBtn}</div></div>`;
    $("#nix-ports-container").append(html);
}
function removePortRow(idx) { $(`#port-row-${idx}`).remove(); }
function handleOverridePortClick() { addPortRow('', '', '', false); }

function populatePortRows(portStr, presetName) {
    $("#nix-ports-container").empty();
    if (!portStr) return;
            var labels = {
        'actual-budget:5006': 'HTTP',
        'adguard-home:3000': 'Setup',
        'adguard-home:53': 'DNS',
        'adguard-home:80': 'Web GUI',
        'agent-dvr:8090': 'HTTP',
        'airsonic-advanced:4040': 'HTTP',
        'anythingllm:3001': 'HTTP',
        'apprise:8000': 'HTTP',
        'appwrite:80': 'HTTP',
        'archivebox:8000': 'HTTP',
        'audiobookshelf:8000': 'HTTP',
        'authelia:9091': 'HTTP',
        'authentik:9000': 'HTTP',
        'authentik:9443': 'HTTPS',
        'baserow:80': 'HTTP',
        'bazarr:6767': 'HTTP',
        'bookstack:80': 'HTTP',
        'caddy:2019': 'Admin',
        'caddy:443': 'HTTPS',
        'caddy:80': 'HTTP',
        'calibre-web:8083': 'HTTP',
        'calibre:8080': 'HTTP',
        'changedetection-io:5000': 'HTTP',
        'clickhouse:8123': 'HTTP',
        'clickhouse:9000': 'Native',
        'cloudbeaver:8978': 'HTTP',
        'code-server:8080': 'HTTP',
        'comfyui:8188': 'Web GUI',
        'crowdsec:8080': 'LAPI',
        'cyberchef:8000': 'HTTP',
        'dashy:4000': 'HTTP',
        'deconz:80': 'HTTP',
        'deluge:58846': 'Daemon',
        'deluge:8112': 'Web GUI',
        'dizquetv:8000': 'HTTP',
        'dokuwiki:80': 'HTTP',
        'dozzle:8080': 'HTTP',
        'duplicacy:3875': 'Web GUI',
        'duplicati:8200': 'Web GUI',
        'emby:8096': 'HTTP',
        'emby:8920': 'HTTPS',
        'emulatorjs:80': 'HTTP',
        'ersatztv:8409': 'HTTP',
        'esphome:6052': 'HTTP',
        'etherpad:9001': 'HTTP',
        'excalidraw:3000': 'HTTP',
        'filebot:5800': 'Web GUI',
        'filerun:80': 'HTTP',
        'filezilla:5800': 'Web GUI',
        'firefly-iii:80': 'HTTP',
        'flame:5005': 'HTTP',
        'flaresolverr:8191': 'HTTP',
        'flowise:3000': 'HTTP',
        'focalboard:8000': 'HTTP',
        'forgejo:22': 'SSH',
        'forgejo:3000': 'HTTP',
        'frigate:5000': 'HTTP',
        'gatus:8080': 'HTTP',
        'ghost:2368': 'HTTP',
        'gitea:22': 'SSH',
        'gitea:3000': 'HTTP',
        'gladys-assistant:80': 'HTTP',
        'glances:61208': 'Web GUI',
        'glitchtip:8000': 'HTTP',
        'goaccess:7890': 'WebSockets',
        'gotify:80': 'HTTP',
        'grafana:3000': 'Web GUI',
        'grocy:80': 'HTTP',
        'guacamole:8080': 'HTTP',
        'handbrake:5800': 'Web GUI',
        'headscale:8080': 'HTTP',
        'hedgedoc:3000': 'HTTP',
        'heimdall:443': 'HTTPS',
        'heimdall:80': 'HTTP',
        'hoarder:3000': 'HTTP',
        'homarr:7575': 'HTTP',
        'home-assistant:8123': 'Web GUI',
        'homebridge:8581': 'Web GUI',
        'homepage:3000': 'HTTP',
        'immich:2283': 'HTTP',
        'influxdb:8086': 'API',
        'it-tools:80': 'HTTP',
        'jackett:9117': 'HTTP',
        'jdownloader2:5800': 'Web GUI',
        'jeedom:80': 'HTTP',
        'jellyfin:1900': 'DLNA (UDP)',
        'jellyfin:7359': 'Discovery (UDP)',
        'jellyfin:8096': 'HTTP',
        'jellyfin:8920': 'HTTPS',
        'jellyseerr:5055': 'HTTP',
        'jellystat:3000': 'HTTP',
        'joplin-server:22300': 'HTTP',
        'kanboard:80': 'HTTP',
        'kavita:5000': 'HTTP',
        'kerberos-io:80': 'HTTP',
        'komga:8080': 'HTTP',
        'kopia:52000': 'Web GUI',
        'krusader:5800': 'Web GUI',
        'kutt:3000': 'HTTP',
        'lancache:443': 'HTTPS',
        'lancache:80': 'HTTP',
        'lazylibrarian:5299': 'HTTP',
        'leantime:80': 'HTTP',
        'librechat:3080': 'HTTP',
        'lidarr:8686': 'HTTP',
        'linkding:9090': 'HTTP',
        'linkwarden:3000': 'HTTP',
        'localai:8080': 'HTTP',
        'makemkv:5800': 'Web GUI',
        'mariadb:3306': 'MySQL Port',
        'mastodon:3000': 'Web',
        'mastodon:4000': 'Streaming',
        'matomo:80': 'HTTP',
        'matrix-synapse:8008': 'HTTP',
        'mealie:9000': 'HTTP',
        'mediawiki:80': 'HTTP',
        'mosquitto:1883': 'MQTT',
        'mosquitto:9001': 'WebSockets',
        'mylar3:8090': 'HTTP',
        'mysql:3306': 'MySQL Port',
        'navidrome:4533': 'HTTP',
        'netbird:443': 'HTTPS',
        'netbird:80': 'HTTP',
        'netdata:19999': 'Web GUI',
        'nextcloud:80': 'HTTP',
        'nginx-proxy-manager:443': 'HTTPS',
        'nginx-proxy-manager:80': 'HTTP',
        'nginx-proxy-manager:81': 'Admin Web',
        'nocodb:8080': 'HTTP',
        'node-red:1880': 'HTTP',
        'ntfy:80': 'HTTP',
        'nzbget:6789': 'HTTP',
        'ollama:11434': 'API',
        'ombi:3579': 'HTTP',
        'open-webui:8080': 'Web GUI',
        'openhab:8080': 'HTTP',
        'openhab:8443': 'HTTPS',
        'organizr:80': 'HTTP',
        'outline:3000': 'HTTP',
        'overseerr:5055': 'HTTP',
        'owncloud:80': 'HTTP',
        'paperless-ngx:8000': 'HTTP',
        'petio:7777': 'HTTP',
        'pgadmin4:443': 'HTTPS',
        'pgadmin4:80': 'HTTP',
        'photoprism:2342': 'HTTP',
        'pi-hole:53': 'DNS',
        'pi-hole:80': 'Web GUI',
        'plex:32400': 'PMS HTTP',
        'portainer:9000': 'HTTP',
        'portainer:9443': 'HTTPS',
        'postgresql:5432': 'Postgres Port',
        'prometheus:9090': 'Web GUI',
        'prowlarr:9696': 'HTTP',
        'pterodactyl:443': 'HTTPS',
        'pterodactyl:80': 'HTTP',
        'pufferpanel:8080': 'HTTP',
        'pyload:8000': 'HTTP',
        'pymedusa:8081': 'HTTP',
        'qbittorrent-vpn:6881': 'BT Port',
        'qbittorrent-vpn:8080': 'Web GUI',
        'qbittorrent:6881': 'BT Port',
        'qbittorrent:8080': 'Web GUI',
        'radarr:7878': 'HTTP',
        'readarr:8787': 'HTTP',
        'redis:6379': 'Redis Port',
        'requestrr:4545': 'HTTP',
        'romm:8080': 'HTTP',
        'sabnzbd:8080': 'Web GUI',
        'scrutiny:8080': 'HTTP',
        'scrypted:10443': 'HTTPS Web',
        'seafile:8000': 'HTTP',
        'sentry:9000': 'HTTP',
        'shinobi:8080': 'HTTP',
        'shiori:8080': 'HTTP',
        'shlink:8080': 'HTTP',
        'shoko-server:8111': 'HTTP',
        'sickchill:8081': 'HTTP',
        'sonarr:8989': 'HTTP',
        'speedtest-tracker:80': 'HTTP',
        'sqlite-web:8080': 'HTTP',
        'stable-diffusion-webui:7860': 'Web GUI',
        'statping-ng:8080': 'HTTP',
        'stirling-pdf:8080': 'HTTP',
        'supabase:8000': 'HTTP',
        'swag:443': 'HTTPS',
        'swag:80': 'HTTP',
        'syncthing:21027': 'Local Discovery (UDP)',
        'syncthing:22000': 'Sync Protocol',
        'syncthing:8384': 'Web GUI',
        'tasmoadmin:80': 'HTTP',
        'tautulli:8181': 'HTTP',
        'tdarr:8265': 'Web GUI',
        'tdarr:8266': 'Server',
        'text-generation-webui:7860': 'Web GUI',
        'threadfin:34400': 'HTTP',
        'traefik:443': 'HTTPS',
        'traefik:80': 'HTTP',
        'traefik:8080': 'Admin',
        'transmission-openvpn:9091': 'RPC/Web',
        'transmission:51413': 'Peer Port',
        'transmission:9091': 'RPC/Web',
        'trilium-notes:8080': 'HTTP',
        'tubearchivist:8000': 'HTTP',
        'uboquity:2202': 'HTTP',
        'uboquity:2203': 'Admin',
        'unifi:8080': 'Inform Port',
        'unifi:8443': 'HTTPS GUI',
        'unmanic:8888': 'Web GUI',
        'uptime-kuma:3001': 'Web GUI',
        'vaultwarden:80': 'HTTP',
        'vikunja:3456': 'HTTP',
        'wallabag:80': 'HTTP',
        'wekan:80': 'HTTP',
        'whisparr:6969': 'HTTP',
        'wiki-js:3000': 'HTTP',
        'wireguard-ui:5000': 'Web GUI',
        'wled:80': 'HTTP',
        'wordpress:80': 'HTTP',
        'xteve:34400': 'HTTP',
        'yourls:80': 'HTTP',
        'zigbee2mqtt:8080': 'Web GUI',
        'zoneminder:80': 'HTTP',
        'zwavejs2mqtt:8091': 'Web GUI'
    };
    portStr.split(',').forEach(function(part) {
        part = part.trim();
        if (!part) return;
        var subparts = part.split(':');
        var host = subparts[0];
        var container = subparts.length > 1 ? subparts[1] : subparts[0];
        var label = labels[presetName + ':' + container] || '';
        addPortRow(host, container, label, false);
    });
}

function updatePresetInfo() {
    var uri = $("#custom-uri").val() || "";
    var name = uri.replace("nixpkgs#", "").toLowerCase().trim();
    var infoBox = $("#nix-preset-info-box");
    if (!infoBox.length) {
        $("#custom-uri").parent().after('<div id="nix-preset-info-box" style="margin-top: 10px; display: none;"></div>');
        infoBox = $("#nix-preset-info-box");
    }
            var presets = {
        "actual-budget": "Actual Budget",
        "adguard-home": "AdGuard Home",
        "agent-dvr": "Agent DVR",
        "airsonic-advanced": "Airsonic Advanced",
        "anythingllm": "AnythingLLM",
        "apprise": "Apprise",
        "appwrite": "Appwrite",
        "archivebox": "ArchiveBox",
        "audiobookshelf": "Audiobookshelf",
        "authelia": "Authelia",
        "authentik": "Authentik",
        "baserow": "Baserow",
        "bazarr": "Bazarr",
        "bookstack": "BookStack",
        "caddy": "Caddy",
        "calibre": "Calibre",
        "calibre-web": "Calibre-Web",
        "changedetection-io": "ChangeDetection.io",
        "clickhouse": "ClickHouse",
        "cloudbeaver": "CloudBeaver",
        "cloudflare-ddns": "Cloudflare DDNS",
        "cloudflared": "Cloudflare Tunnel",
        "code-server": "Code-server",
        "comfyui": "ComfyUI",
        "crowdsec": "CrowdSec",
        "cyberchef": "CyberChef",
        "dashy": "Dashy",
        "ddclient": "DDclient",
        "deconz": "deCONZ",
        "deluge": "Deluge",
        "dizquetv": "DizqueTV",
        "dokuwiki": "DokuWiki",
        "doplarr": "Doplarr",
        "dozzle": "Dozzle",
        "duckdns": "DuckDNS",
        "duplicacy": "Duplicacy",
        "duplicati": "Duplicati",
        "emby": "Emby",
        "emulatorjs": "EmulatorJS",
        "ersatztv": "ErsatzTV",
        "esphome": "ESPHome",
        "etherpad": "Etherpad",
        "excalidraw": "Excalidraw",
        "fail2ban": "Fail2ban",
        "filebot": "FileBot",
        "filerun": "Filerun",
        "filezilla": "FileZilla",
        "firefly-iii": "Firefly III",
        "flame": "Flame",
        "flaresolverr": "Flaresolverr",
        "flowise": "Flowise",
        "focalboard": "Focalboard",
        "forgejo": "Forgejo",
        "frigate": "Frigate NVR",
        "gatus": "Gatus",
        "ghost": "Ghost",
        "gitea": "Gitea",
        "gladys-assistant": "Gladys Assistant",
        "glances": "Glances",
        "glitchtip": "GlitchTip",
        "goaccess": "GoAccess",
        "gotify": "Gotify",
        "grafana": "Grafana",
        "grocy": "Grocy",
        "guacamole": "Apache Guacamole",
        "handbrake": "HandBrake",
        "headscale": "Headscale",
        "hedgedoc": "HedgeDoc",
        "heimdall": "Heimdall",
        "hoarder": "Hoarder",
        "homarr": "Homarr",
        "home-assistant": "Home Assistant",
        "homebridge": "Homebridge",
        "homepage": "Homepage",
        "immich": "Immich",
        "influxdb": "InfluxDB",
        "it-tools": "IT-Tools",
        "jackett": "Jackett",
        "jdownloader2": "JDownloader 2",
        "jeedom": "Jeedom",
        "jellyfin": "Jellyfin",
        "jellyseerr": "Jellyseerr",
        "jellystat": "Jellystat",
        "joplin-server": "Joplin Server",
        "kanboard": "Kanboard",
        "kavita": "Kavita",
        "kerberos-io": "Kerberos.io",
        "kometa": "Kometa",
        "komga": "Komga",
        "kopia": "Kopia",
        "krusader": "Krusader",
        "kutt": "Kutt",
        "lancache": "Lancache",
        "lazylibrarian": "LazyLibrarian",
        "leantime": "Leantime",
        "librechat": "LibreChat",
        "lidarr": "Lidarr",
        "linkding": "Linkding",
        "linkwarden": "Linkwarden",
        "localai": "LocalAI",
        "makemkv": "MakeMKV",
        "mariadb": "MariaDB",
        "mastodon": "Mastodon",
        "matomo": "Matomo",
        "matrix-synapse": "Matrix Synapse",
        "mealie": "Mealie",
        "mediawiki": "MediaWiki",
        "mosquitto": "Mosquitto",
        "mylar3": "Mylar 3",
        "mysql": "MySQL",
        "navidrome": "Navidrome",
        "netbird": "Netbird",
        "netdata": "Netdata",
        "nextcloud": "Nextcloud",
        "nginx-proxy-manager": "Nginx Proxy Manager",
        "nocodb": "NocoDB",
        "node-red": "Node-RED",
        "ntfy": "NTFY",
        "nzbget": "NZBGet",
        "ollama": "Ollama",
        "ombi": "Ombi",
        "open-webui": "Open WebUI",
        "openhab": "OpenHAB",
        "organizr": "Organizr",
        "outline": "Outline",
        "overseerr": "Overseerr",
        "owncloud": "OwnCloud",
        "paperless-ngx": "Paperless-ngx",
        "petio": "Petio",
        "pgadmin4": "pgAdmin4",
        "photoprism": "PhotoPrism",
        "pi-hole": "Pi-hole",
        "plex": "Plex Media Server",
        "plex-anisync": "Plex-Anisync",
        "portainer": "Portainer",
        "postgresql": "PostgreSQL",
        "prometheus": "Prometheus",
        "prowlarr": "Prowlarr",
        "pterodactyl": "Pterodactyl",
        "pufferpanel": "PufferPanel",
        "pyload": "PyLoad",
        "pymedusa": "PyMedusa",
        "qbittorrent": "qBittorrent",
        "qbittorrent-vpn": "qBittorrent VPN",
        "radarr": "Radarr",
        "rclone": "Rclone",
        "readarr": "Readarr",
        "recyclarr": "Recyclarr",
        "redis": "Redis",
        "requestrr": "Requestrr",
        "romm": "RomM",
        "sabnzbd": "SABnzbd",
        "scrutiny": "Scrutiny",
        "scrypted": "Scrypted",
        "seafile": "Seafile",
        "sentry": "Sentry",
        "shinobi": "Shinobi",
        "shiori": "Shiori",
        "shlink": "Shlink",
        "shoko-server": "Shoko Server",
        "sickchill": "SickChill",
        "singlefile": "SingleFile",
        "sonarr": "Sonarr",
        "speedtest-tracker": "Speedtest Tracker",
        "sqlite-web": "SQLite-Web",
        "stable-diffusion-webui": "Stable Diffusion WebUI",
        "statping-ng": "Statping-ng",
        "stirling-pdf": "Stirling-PDF",
        "supabase": "Supabase",
        "swag": "SWAG",
        "syncthing": "Syncthing",
        "tasmoadmin": "TasmoAdmin",
        "tautulli": "Tautulli",
        "tdarr": "Tdarr",
        "telegraf": "Telegraf",
        "text-generation-webui": "Text Generation WebUI",
        "threadfin": "Threadfin",
        "traefik": "Traefik",
        "transmission": "Transmission",
        "transmission-openvpn": "Transmission VPN",
        "trilium-notes": "Trilium Notes",
        "tubearchivist": "Tube Archivist",
        "uboquity": "Uboquity",
        "unifi": "UniFi Network Application",
        "unmanic": "Unmanic",
        "uptime-kuma": "Uptime Kuma",
        "vaultwarden": "Vaultwarden",
        "vikunja": "Vikunja",
        "wallabag": "Wallabag",
        "wekan": "Wekan",
        "whisparr": "Whisparr",
        "wiki-js": "Wiki.js",
        "wireguard-ui": "Wireguard-UI",
        "wled": "WLED",
        "wordpress": "WordPress",
        "xteve": "xTeVe",
        "yourls": "YOURLS",
        "zigbee2mqtt": "Zigbee2MQTT",
        "zoneminder": "ZoneMinder",
        "zwavejs2mqtt": "ZwaveJS2MQTT"
    };
    var matched = presets[name];
    if (matched) {
        infoBox.html(`<div style="background: rgba(0, 161, 255, 0.05); border: 1px solid #00a1ff; border-radius: 4px; padding: 12px; font-size: 12px; color: #eee; margin-bottom: 15px;">` +
            `<div style="font-weight: 600; color: #00a1ff; margin-bottom: 4px;"><i class="fa fa-info-circle"></i> Service Preset Detected: ${matched}</div>` +
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
