window.initInstallForm = function() {
    $("#custom-uri").val("").prop('readonly', false);
    $("#custom-appdata").val("");
    $("#custom-puid").val("99");
    $("#custom-pgid").val("100");
    $("#nix-ports-container").empty();
    $("#custom-bind-address").val("0.0.0.0");
    $("#nix-extra-binds-container").empty();
    $("#nix-env-vars-container").empty();
    $("#custom-compile-locally").prop('checked', false);
    $("#custom-command-override").val("");
    $("#custom-network-isolation").prop('checked', false);
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
        $("#custom-network-isolation").prop('checked', editData.network_isolation === '1' || editData.network_isolation === 'true' || editData.network_isolation === 'yes');
        
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
        if (editData.env_vars) {
            try {
                var envs = typeof editData.env_vars === 'string' ? JSON.parse(editData.env_vars) : editData.env_vars;
                if (envs && typeof envs === 'object') {
                    Object.keys(envs).forEach(function(k) {
                        addEnvVarRow(k, envs[k]);
                    });
                }
            } catch(e) {}
        }
        if (editData.compile_locally === '1' || editData.compile_locally === true || editData.compile_locally === 'true') {
            $("#custom-compile-locally").prop('checked', true);
        }
        if (editData.command_override) {
            $("#custom-command-override").val(editData.command_override);
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
    container.html('<div style="font-size: 11px; color: var(--nix-text-secondary); font-style: italic;" id="nix-gpus-loading">Scanning host for GPU devices...</div>');
    
    $.getJSON('/plugins/nix/api.php?action=detect-gpus', function(gpus) {
        container.empty();
        if (!gpus || gpus.length === 0) {
            container.html('<div style="font-size: 12px; color: var(--nix-text-secondary);">No GPU devices detected on host.</div>');
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
                `<span style="color: var(--nix-text-primary);">${gpu.name}</span>` +
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
        var networkIsolation = $("#custom-network-isolation").is(":checked") ? "1" : "0";
        Object.assign(params, {
            appdata: $("#custom-appdata").val(), media: '', puid: $("#custom-puid").val(), pgid: $("#custom-pgid").val(),
            gpu: gpuVal, gpus: selectedGpus, bind_address: $("#custom-bind-address").val(),
            network_isolation: networkIsolation
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

        var envVars = {};
        $(".nix-env-row").each(function() {
            var k = $(this).find(".nix-env-key").val().trim();
            var v = $(this).find(".nix-env-val").val().trim();
            if (k) { envVars[k] = v; }
        });
        params.env_vars = JSON.stringify(envVars);
        params.compile_locally = $("#custom-compile-locally").is(":checked") ? "1" : "0";
        params.command_override = $("#custom-command-override").val().trim();
    }
    $.each(params, (k, v) => form.append($('<input>', { type: 'hidden', name: k, value: v })));
    
    $('body').append(form);
    form.submit();
    form.remove();
    
    var timer = setInterval(function() {
        if (popup.closed) {
            clearInterval(timer);
            if (type === 'service') {
                var target = window.location.pathname.toLowerCase().startsWith('/settings') ? '/Settings/Nix' : '/Nix';
                window.location.href = target;
            }
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
        `<i class="fa fa-folder-open nix-folder-picker-btn" style="position: absolute; right: 10px; top: 50%; transform: translateY(-50%); color: var(--nix-text-muted); cursor: pointer;"></i>` +
        `</div><div style="flex: 1;"><input type="text" class="nix-bind-sandbox" placeholder="Sandbox Path (e.g. /downloads)" value="${sandboxVal}"></div>` +
        `<div><button type="button" class="nix-btn" style="color: #e74c3c; border-color: #e74c3c; margin: 0; padding: 8px 12px;" onclick="removeBindRow(${idx})"><i class="fa fa-times"></i></button></div></div>`;
    $("#nix-extra-binds-container").append(html);
    if (typeof $.fn.fileTreeAttach === 'function') { $(`#bind-host-${idx}`).fileTreeAttach(); }
}
function removeBindRow(idx) { $(`#bind-row-${idx}`).remove(); }

var portRowIndex = 0;
fnPortText = (v) => v ? `<span style="font-size: 11px; color: var(--nix-text-secondary); margin-left: 5px;">(${v})</span>` : '';
function addPortRow(hostVal, containerVal, labelVal, isPresetPort) {
    hostVal = hostVal || '';
    containerVal = containerVal || '';
    labelVal = labelVal || '';
    isPresetPort = isPresetPort || false;
    var idx = portRowIndex++;
    var readonlyContainer = isPresetPort ? 'readonly style="background: var(--nix-bg-tertiary); color: var(--nix-text-secondary);"' : '';
    var deleteBtn = `<button type="button" class="nix-btn" style="color: #e74c3c; border-color: #e74c3c; margin: 0; padding: 8px 12px;" onclick="removePortRow(${idx})"><i class="fa fa-times"></i></button>`;
    var html = `<div class="nix-form-row nix-port-row" id="port-row-${idx}" style="margin-bottom: 8px; display: flex; gap: 8px; align-items: center;">` +
        `<div style="flex: 1;"><label style="font-size: 11px; margin-bottom: 4px; display: block;">Host Port</label>` +
        `<input type="number" class="nix-port-host" id="port-host-${idx}" value="${hostVal}" placeholder="e.g. 8096" min="1" max="65535" required></div>` +
        `<div style="display: flex; align-items: center; justify-content: center; padding-top: 15px;"><i class="fa fa-arrow-right" style="color: var(--nix-text-secondary);"></i></div>` +
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
        'bitwarden:8000': 'Web GUI',
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
        'dify:5001': 'Web GUI',
        'discourse:3000': 'Web GUI',
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
        'fooocus:7865': 'Web GUI',
        'forgejo:22': 'SSH',
        'forgejo:3000': 'HTTP',
        'friendica:80': 'Web GUI',
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
        'humhub:80': 'Web GUI',
        'immich:2283': 'HTTP',
        'influxdb:8086': 'API',
        'invokeai:9090': 'Web GUI',
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
        'keycloak:8080': 'Web GUI',
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
        'lobe-chat:3210': 'Web GUI',
        'localai:8080': 'HTTP',
        'mailserver:25': 'SMTP Port',
        'makemkv:5800': 'Web GUI',
        'mariadb:3306': 'MySQL Port',
        'mastodon:3000': 'Web',
        'mastodon:4000': 'Streaming',
        'matomo:80': 'HTTP',
        'matrix-synapse:8008': 'HTTP',
        'mattermost:8065': 'Web GUI',
        'mealie:9000': 'HTTP',
        'mediawiki:80': 'HTTP',
        'minecraft-server:25565': 'Game Port',
        'mosquitto:1883': 'MQTT',
        'mosquitto:9001': 'WebSockets',
        'mumble-server:64738': 'Voice Port',
        'mylar3:8090': 'HTTP',
        'mysql:3306': 'MySQL Port',
        'n8n:5678': 'Web GUI',
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
        'openvpn:1194': 'VPN Port',
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
        'pritunl:1195': 'VPN Port',
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
        'resilio-sync:8888': 'Web GUI',
        'rocketchat:3000': 'Web GUI',
        'romm:8080': 'HTTP',
        'rsync:873': 'Rsync Port',
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
        'steamcmd:27015': 'Query Port',
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
        'teamspeak-server:9987': 'Voice Port',
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
        'wireguard:51820': 'VPN Port',
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
        "bitwarden": "Bitwarden",
        "bookstack": "BookStack",
        "borgbackup": "BorgBackup",
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
        "dify": "Dify",
        "discourse": "Discourse",
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
        "fooocus": "Fooocus",
        "forgejo": "Forgejo",
        "friendica": "Friendica",
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
        "humhub": "HumHub",
        "immich": "Immich",
        "influxdb": "InfluxDB",
        "invokeai": "InvokeAI",
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
        "keycloak": "Keycloak",
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
        "lobe-chat": "Lobe Chat",
        "localai": "LocalAI",
        "mailserver": "Mail Server",
        "makemkv": "MakeMKV",
        "mariadb": "MariaDB",
        "mastodon": "Mastodon",
        "matomo": "Matomo",
        "matrix-synapse": "Matrix Synapse",
        "mattermost": "Mattermost",
        "mealie": "Mealie",
        "mediawiki": "MediaWiki",
        "minecraft-server": "Minecraft Server",
        "mosquitto": "Mosquitto",
        "mumble-server": "Mumble Server",
        "mylar3": "Mylar 3",
        "mysql": "MySQL",
        "n8n": "n8n",
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
        "openvpn": "OpenVPN",
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
        "pritunl": "Pritunl",
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
        "resilio-sync": "Resilio Sync",
        "restic": "Restic",
        "rocketchat": "Rocket.Chat",
        "romm": "RomM",
        "rsync": "Rsync",
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
        "steamcmd": "SteamCMD",
        "stirling-pdf": "Stirling-PDF",
        "supabase": "Supabase",
        "swag": "SWAG",
        "syncthing": "Syncthing",
        "tailscale": "Tailscale",
        "tasmoadmin": "TasmoAdmin",
        "tautulli": "Tautulli",
        "tdarr": "Tdarr",
        "teamspeak-server": "TeamSpeak Server",
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
        "wireguard": "WireGuard",
        "wireguard-ui": "Wireguard-UI",
        "wled": "WLED",
        "wordpress": "WordPress",
        "xteve": "xTeVe",
        "yourls": "YOURLS",
        "zigbee2mqtt": "Zigbee2MQTT",
        "zoneminder": "ZoneMinder",
        "zwavejs2mqtt": "ZwaveJS2MQTT"
    };
    var icons = {
        "actual-budget": "fa-server",
        "adguard-home": "fa-shield",
        "agent-dvr": "fa-server",
        "airsonic-advanced": "fa-music",
        "anythingllm": "fa-magic",
        "apprise": "fa-server",
        "appwrite": "fa-server",
        "archivebox": "fa-cloud-upload",
        "audiobookshelf": "fa-book",
        "authelia": "fa-lock",
        "authentik": "fa-lock",
        "baserow": "fa-server",
        "bazarr": "fa-closed-captioning",
        "bitwarden": "fa-lock",
        "bookstack": "fa-file-text-o",
        "borgbackup": "fa-cloud-upload",
        "caddy": "fa-exchange",
        "calibre": "fa-book",
        "calibre-web": "fa-book",
        "changedetection-io": "fa-server",
        "clickhouse": "fa-database",
        "cloudbeaver": "fa-server",
        "cloudflare-ddns": "fa-server",
        "cloudflared": "fa-server",
        "code-server": "fa-server",
        "comfyui": "fa-magic",
        "crowdsec": "fa-lock",
        "cyberchef": "fa-server",
        "dashy": "fa-tachometer",
        "ddclient": "fa-server",
        "deconz": "fa-bolt",
        "deluge": "fa-download",
        "dify": "fa-server",
        "discourse": "fa-comments",
        "dizquetv": "fa-server",
        "dokuwiki": "fa-file-text-o",
        "doplarr": "fa-server",
        "dozzle": "fa-server",
        "duckdns": "fa-server",
        "duplicacy": "fa-cloud-upload",
        "duplicati": "fa-cloud-upload",
        "emby": "fa-play-circle",
        "emulatorjs": "fa-gamepad",
        "ersatztv": "fa-server",
        "esphome": "fa-bolt",
        "etherpad": "fa-file-text-o",
        "excalidraw": "fa-file-text-o",
        "fail2ban": "fa-lock",
        "filebot": "fa-server",
        "filerun": "fa-cloud",
        "filezilla": "fa-folder-open-o",
        "firefly-iii": "fa-server",
        "flame": "fa-tachometer",
        "flaresolverr": "fa-server",
        "flowise": "fa-magic",
        "focalboard": "fa-file-text-o",
        "fooocus": "fa-server",
        "forgejo": "fa-server",
        "friendica": "fa-share-alt",
        "frigate": "fa-server",
        "gatus": "fa-server",
        "ghost": "fa-pencil-square-o",
        "gitea": "fa-server",
        "gladys-assistant": "fa-home",
        "glances": "fa-bar-chart",
        "glitchtip": "fa-server",
        "goaccess": "fa-server",
        "gotify": "fa-server",
        "grafana": "fa-bar-chart",
        "grocy": "fa-server",
        "guacamole": "fa-server",
        "handbrake": "fa-server",
        "headscale": "fa-key",
        "hedgedoc": "fa-file-text-o",
        "heimdall": "fa-tachometer",
        "hoarder": "fa-server",
        "homarr": "fa-tachometer",
        "home-assistant": "fa-home",
        "homebridge": "fa-home",
        "homepage": "fa-tachometer",
        "humhub": "fa-share-alt",
        "immich": "fa-cloud",
        "influxdb": "fa-database",
        "invokeai": "fa-server",
        "it-tools": "fa-server",
        "jackett": "fa-search",
        "jdownloader2": "fa-server",
        "jeedom": "fa-home",
        "jellyfin": "fa-play-circle",
        "jellyseerr": "fa-server",
        "jellystat": "fa-server",
        "joplin-server": "fa-file-text-o",
        "kanboard": "fa-file-text-o",
        "kavita": "fa-server",
        "kerberos-io": "fa-server",
        "keycloak": "fa-lock",
        "kometa": "fa-server",
        "komga": "fa-cloud",
        "kopia": "fa-cloud-upload",
        "krusader": "fa-folder-open-o",
        "kutt": "fa-share-alt",
        "lancache": "fa-gamepad",
        "lazylibrarian": "fa-server",
        "leantime": "fa-file-text-o",
        "librechat": "fa-magic",
        "lidarr": "fa-music",
        "linkding": "fa-bookmark-o",
        "linkwarden": "fa-bookmark-o",
        "lobe-chat": "fa-server",
        "localai": "fa-magic",
        "mailserver": "fa-envelope-o",
        "makemkv": "fa-server",
        "mariadb": "fa-database",
        "mastodon": "fa-share-alt",
        "matomo": "fa-server",
        "matrix-synapse": "fa-comments",
        "mattermost": "fa-comments",
        "mealie": "fa-server",
        "mediawiki": "fa-file-text-o",
        "minecraft-server": "fa-gamepad",
        "mosquitto": "fa-server",
        "mumble-server": "fa-comments",
        "mylar3": "fa-server",
        "mysql": "fa-database",
        "n8n": "fa-server",
        "navidrome": "fa-music",
        "netbird": "fa-key",
        "netdata": "fa-bar-chart",
        "nextcloud": "fa-cloud",
        "nginx-proxy-manager": "fa-exchange",
        "nocodb": "fa-server",
        "node-red": "fa-code-fork",
        "ntfy": "fa-server",
        "nzbget": "fa-download",
        "ollama": "fa-magic",
        "ombi": "fa-server",
        "open-webui": "fa-magic",
        "openhab": "fa-home",
        "openvpn": "fa-key",
        "organizr": "fa-tachometer",
        "outline": "fa-file-text-o",
        "overseerr": "fa-server",
        "owncloud": "fa-cloud",
        "paperless-ngx": "fa-file-text-o",
        "petio": "fa-server",
        "pgadmin4": "fa-server",
        "photoprism": "fa-cloud",
        "pi-hole": "fa-shield",
        "plex": "fa-play-circle",
        "plex-anisync": "fa-play-circle",
        "portainer": "fa-server",
        "postgresql": "fa-database",
        "pritunl": "fa-key",
        "prometheus": "fa-bar-chart",
        "prowlarr": "fa-search",
        "pterodactyl": "fa-gamepad",
        "pufferpanel": "fa-gamepad",
        "pyload": "fa-server",
        "pymedusa": "fa-server",
        "qbittorrent": "fa-download",
        "qbittorrent-vpn": "fa-download",
        "radarr": "fa-film",
        "rclone": "fa-refresh",
        "readarr": "fa-book",
        "recyclarr": "fa-server",
        "redis": "fa-database",
        "requestrr": "fa-server",
        "resilio-sync": "fa-refresh",
        "restic": "fa-cloud-upload",
        "rocketchat": "fa-comments",
        "romm": "fa-gamepad",
        "rsync": "fa-refresh",
        "sabnzbd": "fa-download",
        "scrutiny": "fa-server",
        "scrypted": "fa-bolt",
        "seafile": "fa-cloud",
        "sentry": "fa-server",
        "shinobi": "fa-server",
        "shiori": "fa-bookmark-o",
        "shlink": "fa-server",
        "shoko-server": "fa-server",
        "sickchill": "fa-television",
        "singlefile": "fa-server",
        "sonarr": "fa-television",
        "speedtest-tracker": "fa-server",
        "sqlite-web": "fa-database",
        "stable-diffusion-webui": "fa-magic",
        "statping-ng": "fa-server",
        "steamcmd": "fa-gamepad",
        "stirling-pdf": "fa-file-text-o",
        "supabase": "fa-server",
        "swag": "fa-exchange",
        "syncthing": "fa-refresh",
        "tailscale": "fa-key",
        "tasmoadmin": "fa-server",
        "tautulli": "fa-server",
        "tdarr": "fa-server",
        "teamspeak-server": "fa-comments",
        "telegraf": "fa-server",
        "text-generation-webui": "fa-magic",
        "threadfin": "fa-server",
        "traefik": "fa-exchange",
        "transmission": "fa-download",
        "transmission-openvpn": "fa-download",
        "trilium-notes": "fa-file-text-o",
        "tubearchivist": "fa-server",
        "uboquity": "fa-cloud",
        "unifi": "fa-server",
        "unmanic": "fa-server",
        "uptime-kuma": "fa-bar-chart",
        "vaultwarden": "fa-lock",
        "vikunja": "fa-server",
        "wallabag": "fa-share-alt",
        "wekan": "fa-file-text-o",
        "whisparr": "fa-server",
        "wiki-js": "fa-file-text-o",
        "wireguard": "fa-key",
        "wireguard-ui": "fa-key",
        "wled": "fa-lightbulb-o",
        "wordpress": "fa-pencil-square-o",
        "xteve": "fa-server",
        "yourls": "fa-share-alt",
        "zigbee2mqtt": "fa-bolt",
        "zoneminder": "fa-server",
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
