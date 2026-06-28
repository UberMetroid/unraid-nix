<script>
function serviceAction(name, action) {
    $.post('/plugins/nix/api.php', { action: action, service: name }, function(response) {
        if (response.success) {
            $("#nix-services-list").load('/plugins/nix/api.php?action=render-services');
        } else {
            showNotice("Action failed: " + response.error, "error");
        }
    }, 'json');
}

function editService(name) {
    $.getJSON('/plugins/nix/api.php?action=get-metadata&service=' + name, function(response) {
        if (response.success) {
            sessionStorage.setItem('nix_edit_metadata', JSON.stringify(response.metadata));
            if (window.initInstallForm) {
                window.initInstallForm();
            }
            $('#tab4').trigger('click');
        } else {
            showNotice("Failed to retrieve service configuration: " + response.error, "error");
        }
    });
}

function openLogs(name) {
    var w = 800, h = 600;
    var left = (screen.width/2)-(w/2);
    var top = (screen.height/2)-(h/2);
    window.open('/plugins/nix/api.php?action=logs&service=' + name, 'Service Logs: ' + name, 'toolbar=no, location=no, directories=no, status=no, menubar=no, scrollbars=yes, resizable=yes, copyhistory=no, width='+w+', height='+h+', top='+top+', left='+left);
}

function toggleAutostart(name, checked) {
    $.post('/plugins/nix/api.php', { action: 'toggle-autostart', service: name, enabled: checked }, function(response) {
        if (!response.success) {
            showNotice("Failed to toggle autostart: " + response.error, "error");
        }
        $("#nix-services-list").load('/plugins/nix/api.php?action=render-services');
    }, 'json');
}

function removeService(name) {
    if (!confirm("Are you sure you want to remove service '" + name + "'?\nThis will delete the service from the supervisor list but keep its config directories intact.")) return;
    $.post('/plugins/nix/api.php', { action: 'remove', service: name }, function(response) {
        if (response.success) {
            $("#nix-services-list").load('/plugins/nix/api.php?action=render-services');
        } else {
            showNotice("Failed to remove service: " + response.error, "error");
        }
    }, 'json');
}

$(function() {
    // Check for channel/package updates on load
    $.getJSON('/plugins/nix/api.php?action=check-updates', function(resp) {
        var $status = $("#nix-update-status");
        if (resp.success) {
            if (resp.update_available) {
                $status.html('<span style="color: var(--nix-text-bright); background: rgba(0, 161, 255, 0.12); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--nix-accent); display: inline-flex; align-items: center; gap: 6px;"><i class="fa fa-info-circle"></i> Updates available! <a href="#" onclick="triggerGlobalRebuild(this); return false;" style="color: var(--nix-accent); font-weight: 600; text-decoration: none;">Update All</a></span>');
            } else {
                $status.html('<span style="color: var(--nix-text-muted);"><i class="fa fa-check-circle" style="color: #2ecc71;"></i> Nix channel is up to date</span>');
            }
        } else {
            $status.html('<span style="color: #e74c3c;"><i class="fa fa-exclamation-circle"></i> Update check failed</span>');
        }
    });
});

function triggerGlobalRebuild(link) {
    var $status = $("#nix-update-status");
    $status.html('<span style="color: var(--nix-text-secondary);"><i class="fa fa-spinner fa-spin"></i> Rebuilding all services... (this may take a few minutes)</span>');
    
    $.post('/plugins/nix/api.php', { action: 'global-rebuild' }, function(resp) {
        if (resp.success) {
            showNotice("Global rebuild completed successfully! All services have been upgraded.", "success");
            location.reload();
        } else {
            showNotice("Global rebuild failed: " + resp.error, "error");
            $status.html('<span style="color: #e74c3c;"><i class="fa fa-exclamation-circle"></i> Rebuild failed</span>');
        }
    }, 'json').fail(function() {
        showNotice("Request timed out or failed.", "error");
        $status.html('<span style="color: #e74c3c;"><i class="fa fa-exclamation-circle"></i> Rebuild failed</span>');
    });
}
</script>
