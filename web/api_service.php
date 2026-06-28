<?php
/// Nix Plugin WebGUI PHP API - Service Management Actions
///
/// This file is included by api.php to process active post actions
/// for starting, stopping, configuring, and installing services/packages.

session_set_cookie_params([
    'lifetime' => 0,
    'path' => '/',
    'domain' => '',
    'secure' => isset($_SERVER['HTTPS']),
    'httponly' => true,
    'samesite' => 'Strict',
]);
if (session_status() === PHP_SESSION_NONE) {
    session_start();
}
if (empty($_SESSION['csrf_token'])) {
    session_regenerate_id(true);
    $_SESSION['csrf_token'] = bin2hex(random_bytes(32));
}

// Defense-in-depth: every action in this file is state-changing.
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    if (function_exists('error')) {
        error("This action requires POST.");
    } else {
        http_response_code(405);
        die("This action requires POST.");
    }
}
$csrf_token = $_POST['csrf_token'] ?? '';
$session_csrf = $_SESSION['csrf_token'] ?? '';
if (empty($session_csrf) || !hash_equals($session_csrf, $csrf_token)) {
    if (function_exists('error')) {
        error("Invalid or missing CSRF token.");
    } else {
        http_response_code(403);
        die("Invalid or missing CSRF token.");
    }
}

if ($action === 'start' || $action === 'stop' || $action === 'restart') {
    $service = nix_input_cap($_POST['service'] ?? '', 64, 'service');
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        error("Invalid or missing service name.");
    }
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
    $service = nix_input_cap($_POST['service'] ?? '', 64, 'service');
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        error("Invalid or missing service name.");
    }
    $enabled = isset($_POST['enabled']) ? $_POST['enabled'] : 'false';
    if (!in_array($enabled, ['true', 'false', '1', '0'], true)) {
        error("Invalid enabled value.");
    }
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
    $service = nix_input_cap($_POST['service'] ?? '', 64, 'service');
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        error("Invalid or missing service name.");
    }
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
    $package = nix_input_cap($_POST['package'] ?? '', 256, 'package');
    if (empty($package) || preg_match('/[^a-zA-Z0-9._\-+:]/', $package)) {
        error("Invalid or missing package name.");
    }
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
    $uri = nix_input_cap($_POST['uri'] ?? '', 1024, 'uri');
    if (empty($uri)) {
        error("Missing uri.");
    }
    if (preg_match('/[^a-zA-Z0-9._\-+:\/@#]/', $uri)) {
        error("Invalid uri.");
    }
    $type = isset($_POST['type']) ? $_POST['type'] : '';
    if (!in_array($type, ['cli', 'service'], true)) {
        error("Invalid install type.");
    }
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
        $appdata = nix_input_cap($_POST['appdata'] ?? '', 4096, 'appdata');
        if ($appdata !== '' && preg_match('/(\.\.|\\/\\/)/', $appdata)) {
            error("Invalid appdata path.");
        }
        $media = nix_input_cap($_POST['media'] ?? '', 4096, 'media');
        if ($media !== '' && preg_match('/(\.\.|\\/\\/)/', $media)) {
            error("Invalid media path.");
        }

        $puid_raw = isset($_POST['puid']) ? $_POST['puid'] : '99';
        $pgid_raw = isset($_POST['pgid']) ? $_POST['pgid'] : '100';
        if (!is_numeric($puid_raw) || !is_numeric($pgid_raw)) {
            error("Invalid UID/GID.");
        }
        $puid = max(0, min(65535, intval($puid_raw)));
        $pgid = max(0, min(65535, intval($pgid_raw)));

        $gpu_raw = isset($_POST['gpu']) ? $_POST['gpu'] : '0';
        if (!in_array($gpu_raw, ['0', '1', 'yes', 'no'], true)) {
            error("Invalid gpu value.");
        }
        $gpu = ($gpu_raw === '1' || $gpu_raw === 'yes') ? '1' : '0';

        $gpus = isset($_POST['gpus']) ? $_POST['gpus'] : '';
        if ($gpus !== '' && !preg_match('/^[0-9]+(,[0-9]+)*$/', $gpus)) {
            error("Invalid gpus value.");
        }

        $extra_binds = nix_input_cap($_POST['extra_binds'] ?? '', 8192, 'extra_binds');
        if ($extra_binds !== '') {
            $decoded = json_decode($extra_binds, true);
            if (json_last_error() !== JSON_ERROR_NONE || !is_array($decoded)) {
                error("Invalid extra_binds JSON.");
            }
            foreach ($decoded as $bind) {
                if (!is_array($bind)) {
                    error("Invalid extra_binds entry.");
                }
            }
            if (preg_match('/(\.\.|\\/\\/)/', $extra_binds)) {
                error("Invalid extra_binds path.");
            }
        }

        $port = '';
        $port_raw = isset($_POST['port']) ? $_POST['port'] : '';
        if ($port_raw !== '') {
            if (!ctype_digit((string)$port_raw)) {
                error("Invalid port.");
            }
            $port_int = intval($port_raw);
            if ($port_int < 1 || $port_int > 65535) {
                error("Port out of range.");
            }
            $port = (string)$port_int;
        }

        $bind_address = nix_input_cap($_POST['bind_address'] ?? '', 64, 'bind_address');
        if ($bind_address !== '' && filter_var($bind_address, FILTER_VALIDATE_IP) === false) {
            error("Invalid bind_address.");
        }

        $network_isolation_raw = isset($_POST['network_isolation']) ? $_POST['network_isolation'] : '0';
        if (!in_array($network_isolation_raw, ['0', '1', 'yes', 'no'], true)) {
            error("Invalid network_isolation value.");
        }
        $network_isolation = ($network_isolation_raw === '1' || $network_isolation_raw === 'yes') ? '1' : '0';

        log_debug("Installing background service configs: appdata='{$appdata}', network_isolation='{$network_isolation}', extra_binds='{$extra_binds}'");

        $output = [];
        $code = 0;
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install-service " .
               "--uri " . escapeshellarg($uri) . " " .
               "--appdata " . escapeshellarg($appdata) . " " .
               "--media " . escapeshellarg($media) . " " .
               "--puid " . escapeshellarg((string)$puid) . " " .
               "--pgid " . escapeshellarg((string)$pgid) . " " .
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