$(function() {
    // Check for Nix channel updates asynchronously on load
    checkForUpdates();
});

function checkForUpdates(btn) {
    var statusContainer = $('#nix-update-status');
    statusContainer.html('<span style="color: var(--nix-text-secondary);"><i class="fa fa-refresh fa-spin"></i> Checking for updates...</span>');
    
    $.getJSON('/plugins/nix/api.php?action=check-updates', function(data) {
        if (data.success) {
            var sources = data.sources;
            var tooltipLines = ['=== Checked Nix Sources ==='];
            var numChecked = 0;
            var numUpdates = 0;
            var mainCachedRev = '';
            
            for (var src in sources) {
                if (sources.hasOwnProperty(src)) {
                    numChecked++;
                    var info = sources[src];
                    var statusStr = info.update_available ? ' -> ' + info.latest_rev + ' (UPDATE AVAILABLE)' : ' (up-to-date)';
                    tooltipLines.push(src + ' [' + info.cached_rev + ']' + statusStr);
                    if (info.update_available) {
                        numUpdates++;
                    }
                    if (src === 'nixpkgs') {
                        mainCachedRev = info.cached_rev;
                    }
                }
            }
            
            var tooltipText = tooltipLines.join('\n');
            
            if (data.update_available) {
                statusContainer.html(
                    '<span style="color: #e67e22; font-weight: 500; cursor: help;" title="' + $('<div>').text(tooltipText).html() + '"><i class="fa fa-gift"></i> Update Available (' + numUpdates + ' out of ' + numChecked + ' sources)</span>' +
                    '<button type="button" class="nix-btn-primary" style="margin: 0 5px; padding: 4px 10px; font-size: 11px; font-weight: 600; vertical-align: middle;" onclick="triggerGlobalRebuild(this)">' +
                    '<i class="fa fa-arrow-circle-up"></i> Update & Rebuild' +
                    '</button>' +
                    '<button type="button" class="nix-btn" style="margin: 0; padding: 4px 8px; font-size: 11px; vertical-align: middle;" onclick="checkForUpdates(this)" title="Check again">' +
                    '<i class="fa fa-refresh"></i>' +
                    '</button>'
                );
            } else {
                var flakesText = (numChecked > 1) ? 'Nix channel & ' + (numChecked - 1) + ' flakes checked' : 'Nix channel checked';
                statusContainer.html(
                    '<span style="color: var(--nix-text-secondary); cursor: help;" title="' + $('<div>').text(tooltipText).html() + '"><i class="fa fa-check-circle" style="color: #2ecc71; margin-right: 4px;"></i> ' + flakesText + ': up-to-date (<code>' + mainCachedRev + '</code>)</span>' +
                    '<button type="button" class="nix-btn" style="margin: 0 0 0 8px; padding: 4px 8px; font-size: 11px; vertical-align: middle;" onclick="checkForUpdates(this)" title="Check for updates">' +
                    '<i class="fa fa-refresh"></i> Check' +
                    '</button>'
                );
            }
        } else {
            statusContainer.html('<span style="color: #e74c3c;"><i class="fa fa-exclamation-triangle"></i> Error checking updates</span>');
        }
    }).fail(function() {
        statusContainer.html('<span style="color: #e74c3c;"><i class="fa fa-exclamation-triangle"></i> Network error</span>');
    });
}

function triggerGlobalRebuild(btn) {
    if (!confirm('Are you sure you want to perform a global update and rebuild?\nThis will refresh the Nix channel commits, re-compile packages, and restart all running flakes to apply updates.')) return;
    
    var statusContainer = $('#nix-update-status');
    statusContainer.html('<span style="color: var(--nix-text-secondary);"><i class="fa fa-spinner fa-spin"></i> Rebuilding active services...</span>');
    
    $.post('/plugins/nix/api.php', { action: 'global-rebuild' }, function(response) {
        if (response.success) {
            showNotice('Global update and rebuild completed successfully! All active flakes are running latest packages.', 'success');
            location.reload();
        } else {
            showNotice('Failed to rebuild: ' + response.error, 'error');
            checkForUpdates();
        }
    }, 'json').fail(function() {
        showNotice('Server returned an error while rebuilding.', 'error');
        checkForUpdates();
    });
}
