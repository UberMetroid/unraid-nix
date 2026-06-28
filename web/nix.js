$(function() {
    // All tabs are visible and accessible everywhere.
});

window.nixNavTarget = function(showInNav) {
    if (showInNav) {
        return showInNav === 'yes' ? '/Nix' : '/Settings/Nix';
    }
    return window.location.pathname.toLowerCase().startsWith('/settings') ? '/Settings/Nix' : '/Nix';
};
