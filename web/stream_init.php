<?php
/// Nix Plugin Output Streamer Initialization
///
/// Parses POST parameters and constructs helper command-lines.

set_time_limit(1800);
while (@ob_end_flush());
ob_implicit_flush(true);
header('Content-Type: text/html; charset=utf-8');
header('X-Accel-Buffering: no');

$action = isset($_POST['action']) ? $_POST['action'] : '';
$uri = isset($_POST['uri']) ? $_POST['uri'] : (isset($_POST['package']) ? $_POST['package'] : '');
$type = isset($_POST['type']) ? $_POST['type'] : '';

$args = [];
$args[] = "--action " . escapeshellarg($action);
$args[] = "--uri " . escapeshellarg($uri);
if (!empty($type)) {
    $args[] = "--type " . escapeshellarg($type);
}

$forward_keys = [
    'appdata', 'media', 'puid', 'pgid', 'gpu', 'gpus',
    'extra_binds', 'port', 'bind_address', 'env_vars',
    'network_isolation', 'command_override'
];

foreach ($forward_keys as $key) {
    if (isset($_POST[$key]) && $_POST[$key] !== '') {
        $flag = str_replace('_', '-', $key);
        $args[] = "--" . $flag . " " . escapeshellarg($_POST[$key]);
    }
}

if (isset($_POST['compile_locally']) && $_POST['compile_locally'] === '1') {
    $args[] = "--compile-locally";
}

$cmd = "/usr/local/emhttp/plugins/nix/nix-helper stream-install " . implode(" ", $args);
$title = "Installing Nix Resource: " . htmlspecialchars($uri);
