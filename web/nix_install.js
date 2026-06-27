window.initInstallForm = function() {
    $("#custom-uri").val("").prop('readonly', false);
    $("#custom-appdata").val("");
    $("#custom-puid").val("99");
    $("#custom-pgid").val("100");
    $("#custom-gpu").prop('checked', false);
    $("#nix-ports-container").empty();
    $("#custom-bind-address").val("0.0.0.0");
    $("#nix-extra-binds-container").empty();
    $("#nix-install-section h3").text("Install Flake");
    $("#nix-install-section .nix-subtext").text("Run or daemonize any custom flake from GitHub or a local directory.");
    $("#nix-install-submit-btn").text("Install Flake");

    var editDataStr = sessionStorage.getItem('nix_edit_metadata');
    if (editDataStr) {
        sessionStorage.removeItem('nix_edit_metadata');
        var editData = JSON.parse(editDataStr);
        $("#custom-uri").val(editData.uri).prop('readonly', true);
        $("#custom-appdata").val(editData.appdata);
        $("#custom-puid").val(editData.puid);
        $("#custom-pgid").val(editData.pgid);
        $("#custom-gpu").prop('checked', editData.gpu === '1' || editData.gpu === 'true');
        
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
    updatePresetInfo();
};

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
            gpu: $("#custom-gpu").is(':checked') ? '1' : '0', bind_address: $("#custom-bind-address").val()
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
        'radarr:7878': 'HTTP', 'sonarr:8989': 'HTTP',
        'jellyfin:8096': 'HTTP', 'jellyfin:8920': 'HTTPS', 'jellyfin:1900': 'DLNA (UDP)', 'jellyfin:7359': 'Discovery (UDP)',
        'syncthing:8384': 'Web GUI', 'syncthing:22000': 'Sync Protocol', 'syncthing:21027': 'Local Discovery (UDP)'
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
        "radarr": "Radarr Movie Manager",
        "sonarr": "Sonarr TV Show Manager",
        "jellyfin": "Jellyfin Media Server",
        "syncthing": "Syncthing File Synchronization"
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
