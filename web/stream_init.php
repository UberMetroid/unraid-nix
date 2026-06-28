<?php
/// Nix Plugin Output Streamer Initialization
///
/// Parses POST parameters and constructs helper command-lines.

set_time_limit(120);
while (@ob_end_flush());
ob_implicit_flush(true);
header('Content-Type: text/html; charset=utf-8');
header('X-Accel-Buffering: no');

$action = isset($_POST['action']) ? $_POST['action'] : '';
$cmd = ''; $title = ''; $uri = ''; $timeout_limit = 45; $type = '';

if ($action === 'install-cli') {
    $package = isset($_POST['package']) ? $_POST['package'] : '';
    if (empty($package)) { echo "Missing package name."; exit; }
    $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($package);
    $title = "Installing CLI Package: " . htmlspecialchars($package);
} elseif ($action === 'install-custom') {
    $uri = isset($_POST['uri']) ? $_POST['uri'] : '';
    $type = isset($_POST['type']) ? $_POST['type'] : '';
    if (empty($uri)) { echo "Missing Flake URI."; exit; }
    if ($type === 'cli') {
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($uri);
        $title = "Installing CLI Tool: " . htmlspecialchars($uri);
    } elseif ($type === 'service') {
        $appdata = isset($_POST['appdata']) ? $_POST['appdata'] : '';
        $media = isset($_POST['media']) ? $_POST['media'] : '';
        $puid = isset($_POST['puid']) ? $_POST['puid'] : '99';
        $pgid = isset($_POST['pgid']) ? $_POST['pgid'] : '100';
        $gpu = isset($_POST['gpu']) ? $_POST['gpu'] : '0';
        $gpus = isset($_POST['gpus']) ? $_POST['gpus'] : '';
        $extra_binds = isset($_POST['extra_binds']) ? $_POST['extra_binds'] : '';
        $port = isset($_POST['port']) ? $_POST['port'] : '';
        $bind_address = isset($_POST['bind_address']) ? $_POST['bind_address'] : '';
        $env_vars = isset($_POST['env_vars']) ? $_POST['env_vars'] : '';
        $compile_locally = isset($_POST['compile_locally']) ? $_POST['compile_locally'] : '0';
        $command_override = isset($_POST['command_override']) ? $_POST['command_override'] : '';
        $network_isolation = isset($_POST['network_isolation']) ? $_POST['network_isolation'] : '0';
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install-service " .
               "--uri " . escapeshellarg($uri) . " --appdata " . escapeshellarg($appdata) . " " .
               "--media " . escapeshellarg($media) . " --puid " . escapeshellarg($puid) . " " .
               "--pgid " . escapeshellarg($pgid) . " --gpu " . escapeshellarg($gpu) . " " .
               "--gpus " . escapeshellarg($gpus) . " " .
               "--extra-binds " . escapeshellarg($extra_binds) . " --port " . escapeshellarg($port) . " " .
               "--bind-address " . escapeshellarg($bind_address) . " --env-vars " . escapeshellarg($env_vars) . " " .
               "--network-isolation " . escapeshellarg($network_isolation);
        if (!empty($command_override)) {
            $cmd .= " --command-override " . escapeshellarg($command_override);
        }
        if ($compile_locally === '1') {
            $cmd .= " --compile-locally";
            $timeout_limit = 1800;
            set_time_limit(1800);
        }
        $title = "Installing Nix Service: " . htmlspecialchars($uri);
    } else { echo "Invalid installation type."; exit; }
} else { echo "Unknown action."; exit; }
