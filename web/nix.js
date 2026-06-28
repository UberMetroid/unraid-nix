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
    var toast = document.createElement('div');
    toast.className = 'nix-toast';
    if (type === 'error') toast.classList.add('error');
    else if (type === 'warning') toast.classList.add('warning');
    else toast.classList.add('success');
    var icon = document.createElement('i');
    if (type === 'error') icon.className = 'fa fa-times-circle';
    else if (type === 'warning') icon.className = 'fa fa-exclamation-triangle';
    else icon.className = 'fa fa-check-circle';
    toast.appendChild(icon);
    toast.appendChild(document.createTextNode(' ' + message));
    document.body.appendChild(toast);
    setTimeout(function() {
        toast.style.opacity = '1';
        toast.style.transform = 'translateY(0)';
    }, 50);
    setTimeout(function() {
        toast.style.opacity = '0';
        toast.style.transform = 'translateY(-10px)';
        setTimeout(function() { if (toast.parentNode) toast.parentNode.removeChild(toast); }, 300);
    }, 3000);
};
