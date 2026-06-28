<?php
/// Nix Plugin WebGUI PHP API System Handlers
///
/// Handles daemon service operations, garbage collection, updates checks, Settings saves, etc.

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

header('X-Content-Type-Options: nosniff');

define('NIX_SERVICE_NAME_REGEX', '/^[a-zA-Z0-9_-]+(?:\.[a-zA-Z0-9_-]+)*$/');

// CSRF check only for state-changing actions in this file.
// nix-sys-logs and check-updates are read-only GET-style endpoints and skip this gate.
$csrf_required_actions = [
    'nix-daemon-start', 'nix-daemon-stop', 'nix-daemon-restart',
    'sync-templates', 'save-settings', 'download-diagnostics',
    'collect-garbage', 'global-rebuild',
];
if (in_array($action, $csrf_required_actions, true)) {
    if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
        error("This action requires POST.");
    }
    $csrf_token = $_POST['csrf_token'] ?? '';
    $session_csrf = $_SESSION['csrf_token'] ?? '';
    if (empty($session_csrf) || !hash_equals($session_csrf, $csrf_token)) {
        error("Invalid or missing CSRF token.");
    }
}

if ($action === 'nix-daemon-start') {
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/event/disks_mounted 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'nix-daemon-stop') {
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/event/stopping_svcs 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'nix-daemon-restart') {
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/event/stopping_svcs 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    $output2 = [];
    $code2 = 0;
    exec("/usr/local/emhttp/plugins/nix/event/disks_mounted 2>&1", $output2, $code2);
    if ($code2 !== 0) {
        error(implode("\n", $output2));
    }
    success();
}

if ($action === 'nix-sys-logs') {
    $log_type = isset($_GET['type']) ? $_GET['type'] : 'plugin';
    $file = '';
    if ($log_type === 'plugin') {
        $file = '/var/log/nix-plugin.log';
    } elseif ($log_type === 'compose') {
        $file = '/var/log/nix-process-compose.log';
    } elseif ($log_type === 'daemon') {
        $file = '/var/log/nix-daemon.log';
    } elseif ($log_type === 'gc') {
        $file = '/var/log/nix-gc.log';
    } elseif (strpos($log_type, 'service:') === 0) {
        $service = substr($log_type, 8);
        if (preg_match(NIX_SERVICE_NAME_REGEX, $service)) {
            $file = "/var/log/nix-services/{$service}.log";
            $dir = dirname($file);
            $real_dir = realpath($dir);
            if ($real_dir === false || $real_dir !== '/var/log/nix-services') {
                error("Invalid service log path.");
            }
        } else {
            error("Invalid service log name.");
        }
    } else {
        error("Invalid log type.");
    }
    $lines = isset($_GET['lines']) ? intval($_GET['lines']) : 1000;
    if ($lines <= 0 || $lines > 5000) {
        $lines = 1000;
    }
    $check_services = isset($_GET['check_services']) ? $_GET['check_services'] === '1' : true;
    $service_logs = null;
    if ($check_services) {
        $service_logs = [];
        if (is_dir('/var/log/nix-services')) {
            $files = glob('/var/log/nix-services/*.log');
            if ($files !== false) {
                foreach ($files as $f) {
                    $name = basename($f, '.log');
                    if (preg_match(NIX_SERVICE_NAME_REGEX, $name)) {
                        $service_logs[] = $name;
                    }
                }
            }
        }
        sort($service_logs);
    }

    if (file_exists($file)) {
        if (!is_readable($file)) {
            $res = [
                'success' => true,
                'content' => "Permission Error: Log file is not readable: $file\n(Ensure Unraid system permissions allow access to this file.)"
            ];
            if ($service_logs !== null) {
                $res['service_logs'] = $service_logs;
            }
            echo json_encode($res);
        } else {
            $res = [
                'success' => true, 
                'content' => (string)shell_exec("tail -n " . escapeshellarg($lines) . " " . escapeshellarg($file))
            ];
            if ($service_logs !== null) {
                $res['service_logs'] = $service_logs;
            }
            echo json_encode($res);
        }
    } else {
        $res = [
            'success' => true, 
            'content' => "Log file not found: $file\n(Note: The log file is created once the service starts.)"
        ];
        if ($service_logs !== null) {
            $res['service_logs'] = $service_logs;
        }
        echo json_encode($res);
    }
    exit;
}

if ($action === 'sync-templates') {
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper sync-templates 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'save-settings') {
    $store_path = isset($_POST['store_path']) ? $_POST['store_path'] : '';
    if ($store_path !== '' && (preg_match('/(\.\.|\\/\\/)/', $store_path) || $store_path[0] !== '/')) {
        error("Invalid store_path.");
    }
    $default_appdata_path = isset($_POST['default_appdata_path']) ? $_POST['default_appdata_path'] : '';
    if ($default_appdata_path !== '' && (preg_match('/(\.\.|\\/\\/)/', $default_appdata_path) || $default_appdata_path[0] !== '/')) {
        error("Invalid default_appdata_path.");
    }

    $yes_no = ['yes', 'no'];
    $autostart = isset($_POST['autostart']) ? $_POST['autostart'] : 'yes';
    if (!in_array($autostart, $yes_no, true)) { error("Invalid autostart."); }
    $enable_sandbox = isset($_POST['enable_sandbox']) ? $_POST['enable_sandbox'] : 'no';
    if (!in_array($enable_sandbox, $yes_no, true)) { error("Invalid enable_sandbox."); }
    $show_in_nav = isset($_POST['show_in_nav']) ? $_POST['show_in_nav'] : 'yes';
    if (!in_array($show_in_nav, $yes_no, true)) { error("Invalid show_in_nav."); }
    $allow_source_builds = isset($_POST['allow_source_builds']) ? $_POST['allow_source_builds'] : 'no';
    if (!in_array($allow_source_builds, $yes_no, true)) { error("Invalid allow_source_builds."); }
    $filter_presets_by_hardware = isset($_POST['filter_presets_by_hardware']) ? $_POST['filter_presets_by_hardware'] : 'yes';
    if (!in_array($filter_presets_by_hardware, $yes_no, true)) { error("Invalid filter_presets_by_hardware."); }
    $enable_pid_isolation = isset($_POST['enable_pid_isolation']) ? $_POST['enable_pid_isolation'] : 'yes';
    if (!in_array($enable_pid_isolation, $yes_no, true)) { error("Invalid enable_pid_isolation."); }
    $enable_uts_isolation = isset($_POST['enable_uts_isolation']) ? $_POST['enable_uts_isolation'] : 'yes';
    if (!in_array($enable_uts_isolation, $yes_no, true)) { error("Invalid enable_uts_isolation."); }
    $enable_ipc_isolation = isset($_POST['enable_ipc_isolation']) ? $_POST['enable_ipc_isolation'] : 'yes';
    if (!in_array($enable_ipc_isolation, $yes_no, true)) { error("Invalid enable_ipc_isolation."); }
    $auto_gc = isset($_POST['auto_gc']) ? $_POST['auto_gc'] : 'no';
    if (!in_array($auto_gc, $yes_no, true)) { error("Invalid auto_gc."); }

    $build_cores_raw = isset($_POST['build_cores']) ? $_POST['build_cores'] : '0';
    if (!ctype_digit((string)$build_cores_raw)) { error("Invalid build_cores."); }
    $build_cores = max(0, min(256, intval($build_cores_raw)));

    $build_jobs_raw = isset($_POST['build_jobs']) ? $_POST['build_jobs'] : '0';
    if (!ctype_digit((string)$build_jobs_raw)) { error("Invalid build_jobs."); }
    $build_jobs = max(0, min(256, intval($build_jobs_raw)));

    $gc_min_free_raw = isset($_POST['gc_min_free']) ? $_POST['gc_min_free'] : '5';
    if (!ctype_digit((string)$gc_min_free_raw)) { error("Invalid gc_min_free."); }
    $gc_min_free = max(0, min(100, intval($gc_min_free_raw)));

    $gc_max_free_raw = isset($_POST['gc_max_free']) ? $_POST['gc_max_free'] : '10';
    if (!ctype_digit((string)$gc_max_free_raw)) { error("Invalid gc_max_free."); }
    $gc_max_free = max(0, min(100, intval($gc_max_free_raw)));

    $allowed_channels = ['nixos-unstable', 'nixos-stable', 'nixpkgs-unstable', 'nixpkgs-stable'];
    $nix_channel = isset($_POST['nix_channel']) ? $_POST['nix_channel'] : 'nixos-unstable';
    if (!in_array($nix_channel, $allowed_channels, true)) { error("Invalid nix_channel."); }

    $output = [];
    $code = 0;
    $cmd = "/usr/local/emhttp/plugins/nix/nix-helper save-settings " .
           "--store-path " . escapeshellarg($store_path) . " " .
           "--autostart " . escapeshellarg($autostart) . " " .
           "--enable-sandbox " . escapeshellarg($enable_sandbox) . " " .
           "--show-in-nav " . escapeshellarg($show_in_nav) . " " .
           "--allow-source-builds " . escapeshellarg($allow_source_builds) . " " .
           "--filter-presets-by-hardware " . escapeshellarg($filter_presets_by_hardware) . " " .
           "--enable-pid-isolation " . escapeshellarg($enable_pid_isolation) . " " .
           "--enable-uts-isolation " . escapeshellarg($enable_uts_isolation) . " " .
           "--enable-ipc-isolation " . escapeshellarg($enable_ipc_isolation) . " " .
           "--auto-gc " . escapeshellarg($auto_gc) . " " .
           "--build-cores " . escapeshellarg((string)$build_cores) . " " .
           "--build-jobs " . escapeshellarg((string)$build_jobs) . " " .
           "--gc-min-free " . escapeshellarg((string)$gc_min_free) . " " .
           "--gc-max-free " . escapeshellarg((string)$gc_max_free) . " " .
           "--nix-channel " . escapeshellarg($nix_channel) . " " .
           "--default-appdata-path " . escapeshellarg($default_appdata_path);

    exec($cmd . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'download-diagnostics') {
    require_once __DIR__ . '/api_diagnostics.php';
    exit;
}

if ($action === 'collect-garbage') {
    $output = [];
    $code = 0;
    exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix-collect-garbage -d 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

if ($action === 'check-updates') {
    $meta_dir = '/boot/config/plugins/nix/metadata';
    $uris_to_check = ['nixpkgs'];
    
    if (is_dir($meta_dir)) {
        $meta_files = glob("$meta_dir/*.json");
        if ($meta_files !== false) {
            foreach ($meta_files as $f) {
                $content = file_get_contents($f);
                if ($content) {
                    $meta = json_decode($content, true);
                    if (isset($meta['uri']) && !empty($meta['uri'])) {
                        $uri = $meta['uri'];
                        $source = $uri;
                        if (strpos($uri, '#') !== false) {
                            $parts = explode('#', $uri);
                            $source = $parts[0];
                        }
                        if (!in_array($source, $uris_to_check)) {
                            $uris_to_check[] = $source;
                        }
                    }
                }
            }
        }
    }
    
    $update_available = false;
    $checked_sources = [];
    
    foreach ($uris_to_check as $source) {
        if (preg_match('/[^a-zA-Z0-9:\/\-_#\.\+@]/', $source)) {
            continue;
        }
        
        $output_cached = [];
        $code_cached = 0;
        exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix flake metadata " . escapeshellarg($source) . " --json 2>/dev/null", $output_cached, $code_cached);
        
        $output_refresh = [];
        $code_refresh = 0;
        exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix flake metadata " . escapeshellarg($source) . " --refresh --json 2>/dev/null", $output_refresh, $code_refresh);
        
        if ($code_cached === 0 && $code_refresh === 0) {
            $cached_data = json_decode(implode("\n", $output_cached), true);
            $refresh_data = json_decode(implode("\n", $output_refresh), true);
            
            if ($cached_data && $refresh_data) {
                $cached_rev = isset($cached_data['locked']['rev']) ? $cached_data['locked']['rev'] : (isset($cached_data['revision']) ? $cached_data['revision'] : '');
                $latest_rev = isset($refresh_data['locked']['rev']) ? $refresh_data['locked']['rev'] : (isset($refresh_data['revision']) ? $refresh_data['revision'] : '');
                
                if (!empty($cached_rev) && !empty($latest_rev)) {
                    $checked_sources[$source] = [
                        'cached_rev' => substr($cached_rev, 0, 7),
                        'latest_rev' => substr($latest_rev, 0, 7),
                        'update_available' => ($cached_rev !== $latest_rev)
                    ];
                    if ($cached_rev !== $latest_rev) {
                        $update_available = true;
                    }
                }
            }
        }
    }
    
    echo json_encode([
        'success' => true,
        'update_available' => $update_available,
        'sources' => $checked_sources
    ]);
    exit;
}

if ($action === 'global-rebuild') {
    log_debug("Global update and rebuild requested. Refreshing Nix channel inputs...");
    
    $output = [];
    $code = 0;
    exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix flake metadata nixpkgs --refresh 2>&1", $output, $code);
    if ($code !== 0) {
        error("Failed to refresh nixpkgs channel cache: " . implode("\n", $output));
    }
    
    log_debug("Nix channel refreshed successfully. Restarting process supervisor to compile/rebuild all active services...");
    
    $output_stop = [];
    $code_stop = 0;
    exec("/usr/local/emhttp/plugins/nix/event/stopping_svcs 2>&1", $output_stop, $code_stop);
    
    $output_start = [];
    $code_start = 0;
    exec("/usr/local/emhttp/plugins/nix/event/disks_mounted 2>&1", $output_start, $code_start);
    
    if ($code_stop !== 0 || $code_start !== 0) {
        error("Channel cache updated, but failed to restart Nix supervisor daemon: " . implode("\n", array_merge($output_stop, $output_start)));
    }
    
    log_debug("Global update and rebuild completed successfully.");
    success();
}

error("Unknown API action.");
