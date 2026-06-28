<script>
$(function() {
    if (typeof $.fn.fileTreeAttach === 'function') {
        $("#settings-store-path").fileTreeAttach();
        $("#settings-default-appdata-path").fileTreeAttach();
    }
});

function saveSettings(btn) {
    var $btn = $(btn);
    var originalHtml = $btn.html();
    $btn.prop('disabled', true).html('<i class="fa fa-spinner fa-spin"></i> Saving...');
    
    var path = $("#settings-store-path").val();
    var defaultAppdataPath = $("#settings-default-appdata-path").val();
    var auto = $("#settings-autostart").val();
    var enableSandbox = $("#settings-enable-sandbox").val();
    var enablePidIsolation = $("#settings-enable-pid-isolation").val();
    var enableUtsIsolation = $("#settings-enable-uts-isolation").val();
    var enableIpcIsolation = $("#settings-enable-ipc-isolation").val();
    var autoGc = $("#settings-auto-gc").val();
    var showInNav = $("#settings-show-in-nav").val();
    var allowSourceBuilds = $("#settings-allow-source-builds").val();
    var filterPresetsByHardware = $("#settings-filter-presets-by-hardware").val();
    var buildCores = $("#settings-build-cores").val();
    var buildJobs = $("#settings-build-jobs").val();
    var gcMinFree = $("#settings-gc-min-free").val();
    var gcMaxFree = $("#settings-gc-max-free").val();
    var nixChannel = $("#settings-nix-channel").val();
    
    if (path.indexOf("/boot") === 0) {
        showNotice("Error: Storage location cannot be on your USB flash drive (/boot). Choose a pool disk or array share.", "error");
        $btn.prop('disabled', false).html(originalHtml);
        return;
    }
    
    if (defaultAppdataPath.indexOf("/boot") === 0) {
        showNotice("Error: Default Appdata Path cannot be on your USB flash drive (/boot). Choose a pool disk or array share.", "error");
        $btn.prop('disabled', false).html(originalHtml);
        return;
    }

    if (showInNav === 'no') {
        if (!confirm("Are you sure you want to hide Nix from the main navigation menu?\nThis will remove the Nix tab from the top navigation bar.")) {
            $btn.prop('disabled', false).html(originalHtml);
            return;
        }
    }
    
    $.post('/plugins/nix/api.php', {
        csrf_token: window.csrf_token || '',
        action: 'save-settings',
        store_path: path,
        default_appdata_path: defaultAppdataPath,
        autostart: auto,
        enable_sandbox: enableSandbox,
        enable_pid_isolation: enablePidIsolation,
        enable_uts_isolation: enableUtsIsolation,
        enable_ipc_isolation: enableIpcIsolation,
        auto_gc: autoGc,
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
            showNotice("Settings applied successfully! Restart the Nix services for the changes to take effect.", "success");
            setTimeout(function() {
                var target = typeof window.nixNavTarget === 'function' ? window.nixNavTarget(showInNav) : '/Nix';
                location.href = target;
            }, 1000);
        } else {
            showNotice("Failed to save settings: " + resp.error, "error");
            $btn.prop('disabled', false).html(originalHtml);
        }
    }, 'json').fail(function() {
        showNotice("Server returned an error.", "error");
        $btn.prop('disabled', false).html(originalHtml);
    });
}

function toggleNixDaemon(action) {
    var btnContainer = document.getElementById('nix-daemon-controls');
    btnContainer.innerHTML = '<i class="fa fa-spinner fa-spin"></i> Processing...';
    
    $.post('/plugins/nix/api.php', {
        csrf_token: window.csrf_token || '',
        action: 'nix-daemon-' + action
    }, function(response) {
        if (response.success) {
            location.reload();
        } else {
            showNotice("Action failed: " + response.error, "error");
            setTimeout(function() { location.reload(); }, 1500);
        }
    }, 'json');
}

function syncTemplates(btn) {
    var $btn = $(btn);
    if ($btn.prop('disabled')) return;
    
    var originalHtml = $btn.html();
    $btn.prop('disabled', true).html('<i class="fa fa-spinner fa-spin"></i> Syncing...');
    
    $.post('/plugins/nix/api.php', {
        csrf_token: window.csrf_token || '',
        action: 'sync-templates'
    }, function(response) {
        if (response.success) {
            showNotice("Templates successfully updated!", "success");
            setTimeout(function() { location.reload(); }, 1000);
        } else {
            showNotice("Failed to sync templates: " + response.error, "error");
            $btn.prop('disabled', false).html(originalHtml);
        }
    }, 'json').fail(function() {
        showNotice("Server returned an error while syncing.", "error");
        $btn.prop('disabled', false).html(originalHtml);
    });
}

function collectGarbage(btn) {
    var $btn = $(btn);
    if ($btn.prop('disabled')) return;
    
    var originalHtml = $btn.html();
    $btn.prop('disabled', true).html('<i class="fa fa-spinner fa-spin"></i> Cleaning...');
    
    $.post('/plugins/nix/api.php', {
        csrf_token: window.csrf_token || '',
        action: 'collect-garbage'
    }, function(response) {
        if (response.success) {
            showNotice("Garbage collection complete!", "success");
            $btn.prop('disabled', false).html(originalHtml);
        } else {
            showNotice("Failed to collect garbage: " + response.error, "error");
            $btn.prop('disabled', false).html(originalHtml);
        }
    }, 'json').fail(function() {
        showNotice("Server returned an error while collecting garbage.", "error");
        $btn.prop('disabled', false).html(originalHtml);
    });
}
</script>
