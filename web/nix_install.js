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
        showNotice("Configuration Location is required for services.", "warning");
        return;
    }
    
    var selectedGpus = $(".nix-gpu-checkbox:checked").map(function() { return this.value; }).get().join(',');
    var gpuVal = selectedGpus ? '1' : '0';
    var width = 900, height = 600;
    var left = (window.screen.width - width) / 2, top = (window.screen.height - height) / 2;
    var popupName = 'nix_install_popup_' + Date.now();
    var popup = window.open('', popupName, `scrollbars=yes,resizable=yes,status=no,location=no,toolbar=no,menubar=no,width=${width},height=${height},left=${left},top=${top}`);
    
    if (!popup) {
        showNotice("Popup blocker prevented opening the installation console. Please allow popups for this site.", "warning");
        return;
    }
    
    var form = $('<form>', { method: 'POST', action: '/plugins/nix/stream.php', target: popupName });
    var params = { csrf_token: window.csrf_token || '', action: 'install-custom', uri: uri, type: type };
    if (type === 'service') {
        var networkIsolation = $("#custom-network-isolation").is(":checked") ? "1" : "0";
        Object.assign(params, {
            appdata: $("#custom-appdata").val(), media: '', puid: $("#custom-puid").val(), pgid: $("#custom-pgid").val(),
            gpu: gpuVal, gpus: selectedGpus, bind_address: $("#custom-bind-address").val(), network_isolation: networkIsolation
        });
        params.port = $(".nix-port-row").map(function() {
            var host = $(this).find(".nix-port-host").val();
            var container = $(this).find(".nix-port-container").val();
            return (host && container) ? host + ":" + container : null;
        }).get().join(',');
        params.extra_binds = JSON.stringify($(".nix-bind-row").map(function() {
            var host = $(this).find(".nix-bind-host").val();
            var sandbox = $(this).find(".nix-bind-sandbox").val();
            return (host && sandbox) ? { host: host, sandbox: sandbox } : null;
        }).get());
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
                var target = typeof window.nixNavTarget === 'function' ? window.nixNavTarget() : '/Nix';
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
