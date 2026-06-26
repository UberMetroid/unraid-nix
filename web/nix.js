$(function() {
    // Check if the Nix top navigation tab is present in the DOM
    var hasTopNavTab = $('#menu a[href="/Nix"]').length > 0;
    var isSettingsPage = window.location.pathname.toLowerCase().startsWith('/settings/nix');
    
    // Find all tabs
    $('.tabs-container button').each(function(index) {
        var isSettingTab = (index === 3 || index === 4);
        
        if (hasTopNavTab) {
            // When top navigation tab is enabled, split them up
            if (isSettingsPage) {
                // Under /Settings/Nix, hide management tabs (index 0, 1, 2)
                if (!isSettingTab) {
                    $(this).hide();
                }
            } else {
                // Under /Nix, hide settings/logs tabs (index 3, 4)
                if (isSettingTab) {
                    $(this).hide();
                }
            }
        } else {
            // When top navigation tab is disabled, show all tabs under /Settings/Nix
            // (Do nothing, let all tabs be visible)
        }
    });
    
    // Now check if the currently selected tab is hidden
    var activeTab = $('.tabs-container button[aria-selected="true"]');
    if (activeTab.length && activeTab.css('display') === 'none') {
        // Find the first visible tab and click it
        var firstVisibleTab = $('.tabs-container button').filter(function() {
            return $(this).css('display') !== 'none';
        }).first();
        
        if (firstVisibleTab.length) {
            firstVisibleTab.trigger('click');
        }
    }
});
