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
    $allow_source_builds = isset($_POST['allow_source_builds']) ? $_POST['allow_source_builds'] : 'no';
    
    $output = [];
    $code = 0;
    $cmd = "/usr/local/emhttp/plugins/nix/nix-helper save-settings " .
           "--store-path " . escapeshellarg($store_path) . " " .
           "--autostart " . escapeshellarg($autostart) . " " .
           "--enable-sandbox " . escapeshellarg($enable_sandbox) . " " .
           "--enable-cli " . escapeshellarg($enable_cli) . " " .
           "--show-in-nav " . escapeshellarg($show_in_nav) . " " .
           "--allow-source-builds " . escapeshellarg($allow_source_builds);
           
    exec($cmd . " 2>&1", $output, $code);
    if ($code !== 0) {
        error(implode("\n", $output));
    }
    success();
}

error("Unknown API action.");
