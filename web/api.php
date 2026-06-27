<?php
/// Nix Plugin WebGUI PHP API Router
///
/// This script intercepts AJAX actions from the Unraid browser interface,
/// executes subcommands on the compiled Rust helper, and returns JSON or HTML.
header('Content-Type: application/json');

$action = isset($_REQUEST['action']) ? $_REQUEST['action'] : '';

// Helper function to write debug logs
function log_debug($msg) {
    $log_path = '/var/log/nix-plugin.log';
    $now = date('Y-m-d H:i:s');
    file_put_contents($log_path, "$now [DEBUG] $msg\n", FILE_APPEND);
}

log_debug("API Route invoked: action='{$action}', method='{$_SERVER['REQUEST_METHOD']}'");

// Helper function to return JSON error responses
function error($msg) {
    log_debug("API Response Failure: error='{$msg}'");
    echo json_encode(['success' => false, 'error' => $msg]);
    exit;
}

// Helper function to return JSON success responses
function success() {
    log_debug("API Response Success");
    echo json_encode(['success' => true]);
    exit;
}

// 1. Logs viewer (outputs raw HTML, bypasses JSON header)
if ($action === 'logs') {
    header('Content-Type: text/html');
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        echo "Invalid service name.";
        exit;
    }
    passthru("/usr/local/emhttp/plugins/nix/nix-helper view-logs " . escapeshellarg($service));
    exit;
}

// 2. Search packages (outputs HTML directly, bypasses JSON header)
if ($action === 'search') {
    header('Content-Type: text/html');
    $query = isset($_GET['q']) ? $_GET['q'] : '';
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render search " . escapeshellarg($query));
    exit;
}

// 2b. Render services table (outputs HTML directly, bypasses JSON header)
if ($action === 'render-services') {
    header('Content-Type: text/html');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render services 2>&1");
    exit;
}

// 2c. Render presets store grid (outputs HTML directly)
if ($action === 'render-presets') {
    header('Content-Type: text/html');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render presets 2>&1");
    exit;
}

// 2d. Render dashboard widget (outputs HTML directly, bypasses JSON header)
if ($action === 'get_dashboard') {
    header('Content-Type: text/html');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render dashboard 2>/dev/null");
    exit;
}

// 2e. Render dashboard rows as JSON (outputs JSON directly)
if ($action === 'get_dashboard_json') {
    header('Content-Type: application/json');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render dashboard-json 2>/dev/null");
    exit;
}

// 2f. Get service icon (streams image directly)
if ($action === 'get-icon') {
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        header('HTTP/1.1 400 Bad Request');
        exit;
    }

    $path = trim(shell_exec("/usr/local/emhttp/plugins/nix/nix-helper get-icon " . escapeshellarg($service)));
    if (!empty($path) && file_exists($path) && is_file($path)) {
        if (strpos($path, '/nix/store/') === 0) {
            $ext = strtolower(pathinfo($path, PATHINFO_EXTENSION));
            if ($ext === 'svg') {
                header('Content-Type: image/svg+xml');
            } elseif ($ext === 'png') {
                header('Content-Type: image/png');
            } elseif ($ext === 'ico') {
                header('Content-Type: image/x-icon');
            } else {
                header('Content-Type: image/png');
            }
            header('Cache-Control: max-age=86400');
            readfile($path);
            exit;
        }
    }

    header('Content-Type: image/svg+xml');
    header('Cache-Control: max-age=86400');
    echo '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="#00a1ff" width="32" height="32"><path d="M440 256c0 101.6-82.4 184-184 184S72 357.6 72 256s82.4-184 184-184s184 82.4 184 184zm-184-88c-11 0-20 9-20 20v24.8l-17.5-17.5c-7.8-7.8-20.5-7.8-28.3 0s-7.8 20.5 0 28.3l37.8 37.8c3.9 3.9 9 5.9 14.1 5.9s10.2-2 14.1-5.9l37.8-37.8c7.8-7.8 7.8-20.5 0-28.3s-20.5-7.8-28.3 0L280 212.8V188c0-11-9-20-20-20z"/></svg>';
    exit;
}

// 2c. Detect host GPUs (outputs JSON)
if ($action === 'detect-gpus') {
    passthru("/usr/local/emhttp/plugins/nix/nix-helper detect-gpus");
    exit;
}

// 2c. Get service metadata (JSON response)
if ($action === 'get-metadata') {
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        error("Invalid or missing service name.");
    }
    passthru("/usr/local/emhttp/plugins/nix/nix-helper get-metadata " . escapeshellarg($service));
    exit;
}

// 3. Service actions and installation triggers delegated to helper
if (in_array($action, ['start', 'stop', 'restart', 'toggle-autostart', 'remove', 'install-cli', 'install-custom'])) {
    require_once __DIR__ . '/api_service.php';
    exit;
}

// 6. Nix system daemon control (Start/Stop environment)
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

// 7. Fetch Nix Daemon / Process Compose system logs
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
    
    if (file_exists($file)) {
        echo json_encode([
            'success' => true, 
            'content' => htmlspecialchars(shell_exec("tail -n 250 " . escapeshellarg($file)))
        ]);
    } else {
        echo json_encode([
            'success' => true, 
            'content' => "Log file not found: $file\n(Note: The log file is created once the service starts.)"
        ]);
    }
    exit;
}

// 7b. Sync Preset Templates from Templates repository
if ($action === 'sync-templates') {
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper sync-templates 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

// 8. Save Configuration Settings
if ($action === 'save-settings') {
    $store_path = isset($_POST['store_path']) ? $_POST['store_path'] : '';
    $autostart = isset($_POST['autostart']) ? $_POST['autostart'] : 'yes';
    $enable_sandbox = isset($_POST['enable_sandbox']) ? $_POST['enable_sandbox'] : 'no';
    $enable_cli = isset($_POST['enable_cli']) ? $_POST['enable_cli'] : 'no';
    $show_in_nav = isset($_POST['show_in_nav']) ? $_POST['show_in_nav'] : 'yes';
    $allow_source_builds = isset($_POST['allow_source_builds']) ? $_POST['allow_source_builds'] : 'no';
    $filter_presets_by_hardware = isset($_POST['filter_presets_by_hardware']) ? $_POST['filter_presets_by_hardware'] : 'yes';
    $enable_pid_isolation = isset($_POST['enable_pid_isolation']) ? $_POST['enable_pid_isolation'] : 'yes';
    $enable_uts_isolation = isset($_POST['enable_uts_isolation']) ? $_POST['enable_uts_isolation'] : 'yes';
    $enable_ipc_isolation = isset($_POST['enable_ipc_isolation']) ? $_POST['enable_ipc_isolation'] : 'yes';
    $auto_gc = isset($_POST['auto_gc']) ? $_POST['auto_gc'] : 'no';
    $store_quota = isset($_POST['store_quota']) ? $_POST['store_quota'] : '30';
    $build_cores = isset($_POST['build_cores']) ? $_POST['build_cores'] : '0';
    $build_jobs = isset($_POST['build_jobs']) ? $_POST['build_jobs'] : '0';
    $gc_min_free = isset($_POST['gc_min_free']) ? $_POST['gc_min_free'] : '5';
    $gc_max_free = isset($_POST['gc_max_free']) ? $_POST['gc_max_free'] : '10';
    $nix_channel = isset($_POST['nix_channel']) ? $_POST['nix_channel'] : 'nixos-unstable';
    
    $output = [];
    $code = 0;
    $cmd = "/usr/local/emhttp/plugins/nix/nix-helper save-settings " .
           "--store-path " . escapeshellarg($store_path) . " " .
           "--autostart " . escapeshellarg($autostart) . " " .
           "--enable-sandbox " . escapeshellarg($enable_sandbox) . " " .
           "--enable-cli " . escapeshellarg($enable_cli) . " " .
           "--show-in-nav " . escapeshellarg($show_in_nav) . " " .
           "--allow-source-builds " . escapeshellarg($allow_source_builds) . " " .
           "--filter-presets-by-hardware " . escapeshellarg($filter_presets_by_hardware) . " " .
           "--enable-pid-isolation " . escapeshellarg($enable_pid_isolation) . " " .
           "--enable-uts-isolation " . escapeshellarg($enable_uts_isolation) . " " .
           "--enable-ipc-isolation " . escapeshellarg($enable_ipc_isolation) . " " .
           "--auto-gc " . escapeshellarg($auto_gc) . " " .
           "--store-quota " . escapeshellarg($store_quota) . " " .
           "--build-cores " . escapeshellarg($build_cores) . " " .
           "--build-jobs " . escapeshellarg($build_jobs) . " " .
           "--gc-min-free " . escapeshellarg($gc_min_free) . " " .
           "--gc-max-free " . escapeshellarg($gc_max_free) . " " .
           "--nix-channel " . escapeshellarg($nix_channel);
           
    exec($cmd . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

// 9. Generate and Download Diagnostics Bundle
if ($action === 'download-diagnostics') {
    $tmp_dir = '/tmp/nix-diagnostics-' . uniqid();
    if (!mkdir($tmp_dir, 0755, true)) {
        die("Failed to create temporary diagnostics directory.");
    }
    
    $cfg_dir = '/boot/config/plugins/nix';
    if (file_exists("$cfg_dir/nix.cfg")) {
        copy("$cfg_dir/nix.cfg", "$tmp_dir/nix.cfg");
    }
    if (file_exists("$cfg_dir/process-compose.yml")) {
        copy("$cfg_dir/process-compose.yml", "$tmp_dir/process-compose.yml");
    }
    
    $log_files = [
        '/var/log/nix-daemon.log' => 'nix-daemon.log',
        '/var/log/nix-plugin.log' => 'nix-plugin.log',
        '/var/log/nix-process-compose.log' => 'nix-process-compose.log'
    ];
    foreach ($log_files as $src => $dest) {
        if (file_exists($src)) {
            copy($src, "$tmp_dir/$dest");
        }
    }
    
    $services_log_dir = '/var/log/nix-services';
    if (is_dir($services_log_dir)) {
        @mkdir("$tmp_dir/services", 0755);
        $files = glob("$services_log_dir/*.log");
        if ($files !== false) {
            foreach ($files as $file) {
                copy($file, "$tmp_dir/services/" . basename($file));
            }
        }
    }
    
    $sys_info = [];
    $sys_info[] = "=== Unraid OS Version ===";
    if (file_exists('/etc/unraid-version')) {
        $sys_info[] = trim(file_get_contents('/etc/unraid-version'));
    } else {
        $sys_info[] = "Unknown Unraid Version";
    }
    
    $sys_info[] = "\n=== Nix version ===";
    $nix_ver = [];
    exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix --version 2>&1", $nix_ver);
    $sys_info[] = implode("\n", $nix_ver);
    
    $sys_info[] = "\n=== Nix Daemon Service Status ===";
    $daemon_status = [];
    exec("/etc/rc.d/rc.nix-daemon status 2>&1", $daemon_status);
    $sys_info[] = implode("\n", $daemon_status);
    
    $sys_info[] = "\n=== Memory Specs ===";
    $free_mem = [];
    exec("free -h 2>&1", $free_mem);
    $sys_info[] = implode("\n", $free_mem);
    
    $sys_info[] = "\n=== CPU Specs ===";
    $lscpu = [];
    exec("lscpu | grep -E 'Model name|Core\\(s\\) per socket|Thread\\(s\\) per core|CPU\\(s\\):' 2>&1", $lscpu);
    $sys_info[] = implode("\n", $lscpu);
    
    $sys_info[] = "\n=== Installed Flakes ===";
    $meta_dir = '/boot/config/plugins/nix/metadata';
    if (is_dir($meta_dir)) {
        $meta_files = glob("$meta_dir/*.json");
        if ($meta_files !== false) {
            foreach ($meta_files as $f) {
                $sys_info[] = "  - " . basename($f, ".json");
            }
        }
    }
    
    file_put_contents("$tmp_dir/system_info.txt", implode("\n", $sys_info));
    
    $archive_name = 'nix-diagnostics-' . date('Ymd-His') . '.tar.gz';
    $archive_path = "/tmp/$archive_name";
    
    $output = [];
    $code = 0;
    exec("cd /tmp && tar -czf " . escapeshellarg($archive_name) . " -C " . escapeshellarg(basename($tmp_dir)) . " . 2>&1", $output, $code);
    
    exec("rm -rf " . escapeshellarg($tmp_dir));
    
    if ($code !== 0) {
        die("Failed to package diagnostics: " . implode("\n", $output));
    }
    
    if (file_exists($archive_path)) {
        header('Content-Description: File Transfer');
        header('Content-Type: application/gzip');
        header('Content-Disposition: attachment; filename="' . $archive_name . '"');
        header('Expires: 0');
        header('Cache-Control: must-revalidate');
        header('Pragma: public');
        header('Content-Length: ' . filesize($archive_path));
        readfile($archive_path);
        
        unlink($archive_path);
        exit;
    } else {
        die("Diagnostics archive not found.");
    }
}

// 10. Nix Store Garbage Collection
if ($action === 'collect-garbage') {
    $output = [];
    $code = 0;
    exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix-collect-garbage -d 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

error("Unknown API action.");
