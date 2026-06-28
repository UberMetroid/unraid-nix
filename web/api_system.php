<?php
/// Nix Plugin WebGUI PHP API System Handlers
///
/// Handles daemon service operations, garbage collection, updates checks, Settings saves, etc.

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
    } else {
        error("Invalid log type.");
    }
    $lines = isset($_GET['lines']) ? intval($_GET['lines']) : 1000;
    if ($lines <= 0 || $lines > 5000) {
        $lines = 1000;
    }
    
    if (file_exists($file)) {
        if (!is_readable($file)) {
            echo json_encode([
                'success' => true,
                'content' => "Permission Error: Log file is not readable: $file\n(Ensure Unraid system permissions allow access to this file.)"
            ]);
        } else {
            echo json_encode([
                'success' => true, 
                'content' => (string)shell_exec("tail -n " . escapeshellarg($lines) . " " . escapeshellarg($file))
            ]);
        }
    } else {
        echo json_encode([
            'success' => true, 
            'content' => "Log file not found: $file\n(Note: The log file is created once the service starts.)"
        ]);
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
    $autostart = isset($_POST['autostart']) ? $_POST['autostart'] : 'yes';
    $enable_sandbox = isset($_POST['enable_sandbox']) ? $_POST['enable_sandbox'] : 'no';
    $show_in_nav = isset($_POST['show_in_nav']) ? $_POST['show_in_nav'] : 'yes';
    $allow_source_builds = isset($_POST['allow_source_builds']) ? $_POST['allow_source_builds'] : 'no';
    $filter_presets_by_hardware = isset($_POST['filter_presets_by_hardware']) ? $_POST['filter_presets_by_hardware'] : 'yes';
    $enable_pid_isolation = isset($_POST['enable_pid_isolation']) ? $_POST['enable_pid_isolation'] : 'yes';
    $enable_uts_isolation = isset($_POST['enable_uts_isolation']) ? $_POST['enable_uts_isolation'] : 'yes';
    $enable_ipc_isolation = isset($_POST['enable_ipc_isolation']) ? $_POST['enable_ipc_isolation'] : 'yes';
    $auto_gc = isset($_POST['auto_gc']) ? $_POST['auto_gc'] : 'no';
    $build_cores = isset($_POST['build_cores']) ? $_POST['build_cores'] : '0';
    $build_jobs = isset($_POST['build_jobs']) ? $_POST['build_jobs'] : '0';
    $gc_min_free = isset($_POST['gc_min_free']) ? $_POST['gc_min_free'] : '5';
    $gc_max_free = isset($_POST['gc_max_free']) ? $_POST['gc_max_free'] : '10';
    $nix_channel = isset($_POST['nix_channel']) ? $_POST['nix_channel'] : 'nixos-unstable';
    $default_appdata_path = isset($_POST['default_appdata_path']) ? $_POST['default_appdata_path'] : '';
    
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
           "--build-cores " . escapeshellarg($build_cores) . " " .
           "--build-jobs " . escapeshellarg($build_jobs) . " " .
           "--gc-min-free " . escapeshellarg($gc_min_free) . " " .
           "--gc-max-free " . escapeshellarg($gc_max_free) . " " .
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
