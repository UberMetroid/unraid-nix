<script>
$(function() {
    if (typeof $.fn.fileTreeAttach === 'function') {
        $("#settings-store-path").fileTreeAttach();
    }
});

function saveSettings(e) {
    if (e) e.preventDefault();
    
    var submitBtn = $(".nix-btn-primary");
    var originalHtmls = [];
    submitBtn.each(function(i, btn) {
        var $b = $(btn);
        originalHtmls.push($b.html());
        $b.prop('disabled', true).html('<i class="fa fa-spinner fa-spin"></i> Saving...');
    });
    
    var path = $("#settings-store-path").val();
    var auto = $("#settings-autostart").val();
    var enableSandbox = $("#settings-enable-sandbox").val();
    var enablePidIsolation = $("#settings-enable-pid-isolation").val();
    var enableUtsIsolation = $("#settings-enable-uts-isolation").val();
    var enableIpcIsolation = $("#settings-enable-ipc-isolation").val();
    var autoGc = $("#settings-auto-gc").val();
    var storeQuota = $("#settings-store-quota").val();
    var showInNav = $("#settings-show-in-nav").val();
    var allowSourceBuilds = $("#settings-allow-source-builds").val();
    var filterPresetsByHardware = $("#settings-filter-presets-by-hardware").val();
    var buildCores = $("#settings-build-cores").val();
    var buildJobs = $("#settings-build-jobs").val();
    var gcMinFree = $("#settings-gc-min-free").val();
    var gcMaxFree = $("#settings-gc-max-free").val();
    var nixChannel = $("#settings-nix-channel").val();
    
    if (path.indexOf("/boot") === 0) {
        alert("Error: Storage location cannot be on your USB flash drive (/boot). Choose a pool disk or array share.");
        submitBtn.each(function(i, btn) {
            $(btn).prop('disabled', false).html(originalHtmls[i]);
        });
        return;
    }
    
    $.post('/plugins/nix/api.php', {
        action: 'save-settings',
        store_path: path,
        autostart: auto,
        enable_sandbox: enableSandbox,
        enable_pid_isolation: enablePidIsolation,
        enable_uts_isolation: enableUtsIsolation,
        enable_ipc_isolation: enableIpcIsolation,
        auto_gc: autoGc,
        store_quota: storeQuota,
        show_in_nav: showInNav,
        allow_source_builds: allowSourceBuilds,
        filter_presets_by_hardware: filterPresetsByHardware,
        build_cores: buildCores,
        build_jobs: buildJobs,
        gc_min_free: gcMinFree,
        gc_max_free: gcMaxFree,
        nix_channel: nixChannel
    }, function(resp) {
        if (resp.success) {
            alert("Settings applied successfully! Restart the Nix services for the changes to take effect.");
            if (showInNav === 'yes') {
                location.href = '/Nix';
            } else {
                location.href = '/Settings/Nix';
            }
        } else {
            alert("Failed to save settings: " + resp.error);
            submitBtn.each(function(i, btn) {
                $(btn).prop('disabled', false).html(originalHtmls[i]);
            });
        }
    }, 'json').fail(function() {
        alert("Server returned an error.");
        submitBtn.each(function(i, btn) {
            $(btn).prop('disabled', false).html(originalHtmls[i]);
        });
    });
}

function toggleNixDaemon(action) {
    var btnContainer = document.getElementById('nix-daemon-controls');
    btnContainer.innerHTML = '<i class="fa fa-spinner fa-spin"></i> Processing...';
    
    $.post('/plugins/nix/api.php', { action: 'nix-daemon-' + action }, function(response) {
        if (response.success) {
            location.reload();
        } else {
            alert("Action failed: " + response.error);
            location.reload();
        }
    }, 'json');
}

function syncTemplates(btn) {
    var $btn = $(btn);
    if ($btn.prop('disabled')) return;
    
    var originalHtml = $btn.html();
    $btn.prop('disabled', true).html('<i class="fa fa-spinner fa-spin"></i> Syncing...');
    
    $.post('/plugins/nix/api.php', { action: 'sync-templates' }, function(response) {
        if (response.success) {
            alert("Templates successfully updated!");
            location.reload();
        } else {
            alert("Failed to sync templates: " + response.error);
            $btn.prop('disabled', false).html(originalHtml);
        }
    }, 'json').fail(function() {
        alert("Server returned an error while syncing.");
        $btn.prop('disabled', false).html(originalHtml);
    });
}

function collectGarbage(btn) {
    var $btn = $(btn);
    if ($btn.prop('disabled')) return;
    
    var originalHtml = $btn.html();
    $btn.prop('disabled', true).html('<i class="fa fa-spinner fa-spin"></i> Cleaning...');
    
    $.post('/plugins/nix/api.php', { action: 'collect-garbage' }, function(response) {
        if (response.success) {
            alert("Garbage collection complete!");
            $btn.prop('disabled', false).html(originalHtml);
        } else {
            alert("Failed to collect garbage: " + response.error);
            $btn.prop('disabled', false).html(originalHtml);
        }
    }, 'json').fail(function() {
        alert("Server returned an error while collecting garbage.");
        $btn.prop('disabled', false).html(originalHtml);
    });
}

function saveAllSettings(btn) {
    saveSettings();
}
</script>
