<?php
/// Nix Plugin WebGUI PHP API Router
///
/// This script intercepts AJAX actions from the Unraid browser interface,
/// executes subcommands on the compiled Rust helper, and returns JSON or HTML.
header('Content-Type: application/json');

$action = isset($_REQUEST['action']) ? $_REQUEST['action'] : '';

// Helper function to return JSON error responses
function error($msg) {
    echo json_encode(['success' => false, 'error' => $msg]);
    exit;
}

// Helper function to return JSON success responses
function success() {
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

// 2c. Get service metadata (JSON response)
if ($action === 'get-metadata') {
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        error("Invalid or missing service name.");
    }
    passthru("/usr/local/emhttp/plugins/nix/nix-helper get-metadata " . escapeshellarg($service));
    exit;
}

// 3. Service action triggers (Start, Stop, Restart)
if ($action === 'start' || $action === 'stop' || $action === 'restart') {
    $service = isset($_POST['service']) ? $_POST['service'] : '';
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper service " . escapeshellarg($action) . " " . escapeshellarg($service) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

// 3b. Toggle Service Autostart (On/Off)
if ($action === 'toggle-autostart') {
    $service = isset($_POST['service']) ? $_POST['service'] : '';
    $enabled = isset($_POST['enabled']) ? $_POST['enabled'] : 'false';
    $toggle_val = ($enabled === 'true' || $enabled === '1') ? 'on' : 'off';
    
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper autostart " . escapeshellarg($service) . " " . escapeshellarg($toggle_val) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

// 3c. Remove Service
if ($action === 'remove') {
    $service = isset($_POST['service']) ? $_POST['service'] : '';
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper remove-service " . escapeshellarg($service) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

// 4. Install CLI Package
if ($action === 'install-cli') {
    $package = isset($_POST['package']) ? $_POST['package'] : '';
    $output = [];
    $code = 0;
    exec("/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($package) . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

// 5. Install Custom Flake / Preset Service
if ($action === 'install-custom') {
    $uri = isset($_POST['uri']) ? $_POST['uri'] : '';
    $type = isset($_POST['type']) ? $_POST['type'] : '';
    
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
        $extra_binds = isset($_POST['extra_binds']) ? $_POST['extra_binds'] : '';
        $port = isset($_POST['port']) ? $_POST['port'] : '';
        $bind_address = isset($_POST['bind_address']) ? $_POST['bind_address'] : '';
        
        $output = [];
        $code = 0;
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install-service " .
               "--uri " . escapeshellarg($uri) . " " .
               "--appdata " . escapeshellarg($appdata) . " " .
               "--media " . escapeshellarg($media) . " " .
               "--puid " . escapeshellarg($puid) . " " .
               "--pgid " . escapeshellarg($pgid) . " " .
               "--gpu " . escapeshellarg($gpu) . " " .
               "--extra-binds " . escapeshellarg($extra_binds) . " " .
               "--port " . escapeshellarg($port) . " " .
               "--bind-address " . escapeshellarg($bind_address);
               
        exec($cmd . " 2>&1", $output, $code);
        if ($code !== 0) {
            error(implode("\n", $output));
        }
        success();
    }
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

// 7. Fetch Nix Daemon / Process Compose system logs
if ($action === 'nix-sys-logs') {
    $log_type = isset($_GET['type']) ? $_GET['type'] : 'daemon';
    $file = '';
    if ($log_type === 'daemon') {
        $file = '/var/log/nix-daemon.log';
    } elseif ($log_type === 'compose') {
        $file = '/var/log/nix-process-compose.log';
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

// 8. Save Configuration Settings
if ($action === 'save-settings') {
    $store_path = isset($_POST['store_path']) ? $_POST['store_path'] : '';
    $autostart = isset($_POST['autostart']) ? $_POST['autostart'] : 'yes';
    $enable_sandbox = isset($_POST['enable_sandbox']) ? $_POST['enable_sandbox'] : 'no';
    $enable_cli = isset($_POST['enable_cli']) ? $_POST['enable_cli'] : 'no';
    $show_in_nav = isset($_POST['show_in_nav']) ? $_POST['show_in_nav'] : 'yes';
    
    $output = [];
    $code = 0;
    $cmd = "/usr/local/emhttp/plugins/nix/nix-helper save-settings " .
           "--store-path " . escapeshellarg($store_path) . " " .
           "--autostart " . escapeshellarg($autostart) . " " .
           "--enable-sandbox " . escapeshellarg($enable_sandbox) . " " .
           "--enable-cli " . escapeshellarg($enable_cli) . " " .
           "--show-in-nav " . escapeshellarg($show_in_nav);
           
    exec($cmd . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

error("Unknown API action.");
