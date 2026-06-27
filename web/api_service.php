<?php
/// Nix Plugin WebGUI PHP API - Service Management Actions
///
/// This file is included by api.php to process active post actions
/// for starting, stopping, configuring, and installing services/packages.

if ($action === 'start' || $action === 'stop' || $action === 'restart') {
    $service = isset($_POST['service']) ? $_POST['service'] : '';
    log_debug("Service operation triggered: action='{$action}', service='{$service}'");
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper service " . escapeshellarg($action) . " " . escapeshellarg($service) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'toggle-autostart') {
    $service = isset($_POST['service']) ? $_POST['service'] : '';
    $enabled = isset($_POST['enabled']) ? $_POST['enabled'] : 'false';
    $toggle_val = ($enabled === 'true' || $enabled === '1') ? 'on' : 'off';
    log_debug("Autostart configuration updated: service='{$service}', enabled='{$enabled}', value='{$toggle_val}'");
    
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper autostart " . escapeshellarg($service) . " " . escapeshellarg($toggle_val) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'remove') {
    $service = isset($_POST['service']) ? $_POST['service'] : '';
    log_debug("Service removal triggered: service='{$service}'");
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper remove-service " . escapeshellarg($service) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'install-cli') {
    $package = isset($_POST['package']) ? $_POST['package'] : '';
    log_debug("CLI package installation triggered: package='{$package}'");
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($package) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'install-custom') {
    $uri = isset($_POST['uri']) ? $_POST['uri'] : '';
    $type = isset($_POST['type']) ? $_POST['type'] : '';
    log_debug("Custom installation triggered: uri='{$uri}', type='{$type}'");
    
    if ($type === 'cli') {
        $output = [];
        $code = 0;
        exec("/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($uri) . " 2>&1", $output, $code);
        if ($code !== 0) {
            error(implode("\n", $output));
        }
        success();
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
        $network_isolation = isset($_POST['network_isolation']) ? $_POST['network_isolation'] : '0';
        
        log_debug("Installing background service configs: appdata='{$appdata}', network_isolation='{$network_isolation}', extra_binds='{$extra_binds}'");
        
        $output = [];
        $code = 0;
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install-service " .
               "--uri " . escapeshellarg($uri) . " " .
               "--appdata " . escapeshellarg($appdata) . " " .
               "--media " . escapeshellarg($media) . " " .
               "--puid " . escapeshellarg($puid) . " " .
               "--pgid " . escapeshellarg($pgid) . " " .
               "--gpu " . escapeshellarg($gpu) . " " .
               "--gpus " . escapeshellarg($gpus) . " " .
               "--extra-binds " . escapeshellarg($extra_binds) . " " .
               "--port " . escapeshellarg($port) . " " .
               "--bind-address " . escapeshellarg($bind_address) . " " .
               "--network-isolation " . escapeshellarg($network_isolation);
               
        log_debug("Executing installation helper command: {$cmd}");
        exec($cmd . " 2>&1", $output, $code);
        if ($code !== 0) {
            error(implode("\n", $output));
        }
        success();
    }
}
