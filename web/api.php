<?php
/// Nix Plugin WebGUI PHP API Router
///
/// This script intercepts AJAX actions from the Unraid browser interface,
/// executes subcommands on the compiled Rust helper, and returns JSON or HTML.
header('Content-Type: application/json');

$action = isset($_REQUEST['action']) ? $_REQUEST['action'] : '';

// 1. Logs viewer (outputs raw HTML, bypasses JSON header)
if ($action === 'logs') {
    header('Content-Type: text/html');
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        echo "Invalid service name.";
        exit;
    }
    $log_file = "/var/log/nix-services/" . $service . ".log";
    echo "<html><head><title>Logs: $service</title><style>body{background:#111;color:#eee;font-family:monospace;padding:15px;}</style></head><body>";
    echo "<h3>Active console output for: $service</h3>";
    if (file_exists($log_file)) {
        echo "<pre>" . htmlspecialchars(shell_exec("tail -n 200 " . escapeshellarg($log_file))) . "</pre>";
    } else {
        echo "<p class='text-muted'>No logs found. If the service just started, it might take a few seconds to populate.</p>";
    }
    echo "</body></html>";
    exit;
}

// 2. Search packages (outputs HTML directly, bypasses JSON header)
if ($action === 'search') {
    header('Content-Type: text/html');
    $query = isset($_GET['q']) ? $_GET['q'] : '';
    $html = shell_exec("/usr/local/emhttp/plugins/nix/nix-helper render search " . escapeshellarg($query));
    echo $html;
    exit;
}

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
        
        // Parse name from URI (e.g. nixpkgs#radarr -> radarr)
        $name = str_replace('nixpkgs#', '', $uri);
        $name = last(explode('/', $name));
        $name = last(explode(':', $name));
        $name = last(explode('#', $name));
        $name = preg_replace('/[^a-zA-Z0-9_-]/', '', $name);

        // Fetch command string. Checks if it is a preset
        $cmd = "";
        if (in_array(strtolower($name), ['radarr', 'sonarr', 'jellyfin'])) {
            $cmd = shell_exec(format_preset_cmd($name, $appdata, $media, $puid, $pgid, $gpu));
        } else {
            // Build custom bubblewrap command
            $cmd = shell_exec("/usr/local/emhttp/plugins/nix/nix-helper sandbox --name " . escapeshellarg($name) . " --appdata " . escapeshellarg($appdata) . " --media " . escapeshellarg($media) . " --puid " . escapeshellarg($puid) . " --pgid " . escapeshellarg($pgid) . " --cmd " . escapeshellarg("nix run " . $uri));
        }
        
        $output = [];
        $code = 0;
        exec("/usr/local/emhttp/plugins/nix/nix-helper add-service " . escapeshellarg($name) . " " . escapeshellarg(trim($cmd)) . " 2>&1", $output, $code);
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

// 7. Save Configuration Settings
if ($action === 'save-settings') {
    $store_path = isset($_POST['store_path']) ? $_POST['store_path'] : '';
    $autostart = isset($_POST['autostart']) ? $_POST['autostart'] : 'yes';
    
    $cfg_dir = "/boot/config/plugins/nix";
    if (!file_exists($cfg_dir)) {
        mkdir($cfg_dir, 0777, true);
    }
    
    $cfg_content = "NIX_STORE_PATH=\"" . addslashes($store_path) . "\"\n";
    $cfg_content .= "AUTOSTART_FLAKES=\"" . addslashes($autostart) . "\"\n";
    
    if (file_put_contents($cfg_dir . "/nix.cfg", $cfg_content) === false) {
        error("Failed to write nix.cfg to flash drive.");
    }
    success();
}

// Helper methods for string parsing in PHP
function last($arr) {
    return end($arr);
}

function format_preset_cmd($name, $appdata, $media, $puid, $pgid, $gpu) {
    $media_arg = empty($media) ? "-" : $media;
    return "/usr/local/emhttp/plugins/nix/nix-helper preset " . escapeshellarg($name) . " " . escapeshellarg($appdata) . " " . escapeshellarg($media_arg) . " " . escapeshellarg($puid) . " " . escapeshellarg($pgid) . " " . escapeshellarg($gpu);
}

error("Unknown API action.");
