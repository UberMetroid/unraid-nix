$(function() {
    // All tabs are visible and accessible everywhere.
});

window.nixNavTarget = function(showInNav) {
    if (showInNav) {
        return showInNav === 'yes' ? '/Nix' : '/Settings/Nix';
    }
    return window.location.pathname.toLowerCase().startsWith('/settings') ? '/Settings/Nix' : '/Nix';
};

window.showNotice = function(message, type) {
    if (window.top && window.top.eventMessage) {
        window.top.eventMessage('Nix', message, 'nix.png', type || 'info', 3000);
        return;
    }
    var toast = $('<div class="nix-toast"></div>');
    if (type === 'error') {
        toast.addClass('error').html('<i class="fa fa-times-circle"></i> ' + message);
    } else if (type === 'warning') {
        toast.addClass('warning').html('<i class="fa fa-exclamation-triangle"></i> ' + message);
    } else {
        toast.addClass('success').html('<i class="fa fa-check-circle"></i> ' + message);
    }
    $('body').append(toast);
    setTimeout(function() {
        toast.css({ 'opacity': '1', 'transform': 'translateY(0)' });
    }, 50);
    setTimeout(function() {
        toast.css({ 'opacity': '0', 'transform': 'translateY(-10px)' });
        setTimeout(function() { toast.remove(); }, 300);
    }, 3000);
};
