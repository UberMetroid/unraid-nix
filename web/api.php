<?php
/// Nix Plugin WebGUI PHP API Router
///
/// This script intercepts AJAX actions from the Unraid browser interface,
/// executes subcommands on the compiled Rust helper, and returns JSON or HTML.

if (session_status() === PHP_SESSION_NONE) {
    session_start();
}
if (empty($_SESSION['csrf_token'])) {
    $_SESSION['csrf_token'] = bin2hex(random_bytes(32));
}

header('Content-Type: application/json');

if (!defined('NIX_SERVICE_NAME_REGEX')) {
    define('NIX_SERVICE_NAME_REGEX', '/^[a-zA-Z0-9_-]+(?:\.[a-zA-Z0-9_-]+)*$/');
}

$action = isset($_REQUEST['action']) ? $_REQUEST['action'] : '';

// Helper function to write debug logs safely against log injection
function log_debug($msg) {
    $log_path = '/var/log/nix-plugin.log';
    $now = date('Y-m-d H:i:s');
    // Sanitize carriage returns, newlines and brackets to prevent forged lines
    $safe_msg = str_replace(["\n", "\r", "[", "]"], [" ", " ", "(", ")"], $msg);
    file_put_contents($log_path, "$now [DEBUG] $safe_msg\n", FILE_APPEND);
}

// Skip logging for high-frequency or verbose actions to keep debug logs clean
$no_debug_actions = ['get_dashboard', 'get_dashboard_json', 'nix-sys-logs', 'render-services', 'check-updates'];
$safe_action = str_replace(["\n", "\r", "[", "]"], [" ", " ", "(", ")"], $action);
if (!in_array($safe_action, $no_debug_actions)) {
    log_debug("API Route invoked: action='{$safe_action}', method='" . (isset($_SERVER['REQUEST_METHOD']) ? $_SERVER['REQUEST_METHOD'] : 'CLI') . "'");
}

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
    header('X-Content-Type-Options: nosniff');
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || !preg_match(NIX_SERVICE_NAME_REGEX, $service)) {
        echo "Invalid service name.";
        exit;
    }
    passthru("/usr/local/emhttp/plugins/nix/nix-helper view-logs " . escapeshellarg($service));
    exit;
}

// 2. Search packages (outputs HTML directly, bypasses JSON header)
if ($action === 'search') {
    header('Content-Type: text/html');
    header('X-Content-Type-Options: nosniff');
    $query = isset($_GET['q']) ? $_GET['q'] : '';
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render search " . escapeshellarg($query));
    exit;
}

// 2b. Render services table (outputs HTML directly, bypasses JSON header)
if ($action === 'render-services') {
    header('Content-Type: text/html');
    header('X-Content-Type-Options: nosniff');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render services 2>&1");
    exit;
}

// 2c. Render presets store grid (outputs HTML directly)
if ($action === 'render-presets') {
    header('Content-Type: text/html');
    header('X-Content-Type-Options: nosniff');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render presets 2>&1");
    exit;
}

// 2d. Render dashboard widget (outputs HTML directly, bypasses JSON header)
if ($action === 'get_dashboard') {
    header('Content-Type: text/html');
    header('X-Content-Type-Options: nosniff');
    passthru("/usr/local/emhttp/plugins/nix/nix-helper render dashboard 2>/dev/null");
    exit;
}

// 2e. Render dashboard rows as JSON (outputs JSON directly)
if ($action === 'get_dashboard_json') {
    header('Content-Type: application/json');
    header('X-Content-Type-Options: nosniff');
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
        // Resolve symlinks and verify the canonical path is still inside
        // /nix/store. Without this, a helper-returned path like
        // "/nix/store/../../etc/passwd" passes the prefix check and would
        // be served to the browser.
        $real = realpath($path);
        if ($real !== false && strpos($real, '/nix/store/') === 0) {
            $ext = strtolower(pathinfo($real, PATHINFO_EXTENSION));
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
            readfile($real);
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

// CSRF check for state-changing actions delegated to api_service.php / api_system.php.
// Read-only actions (logs, search, render-*, get-icon, detect-gpus, get-metadata,
// nix-sys-logs, check-updates) exit() above and never reach this block.
$csrf_required_actions = [
    'start', 'stop', 'restart', 'toggle-autostart', 'remove', 'install-cli', 'install-custom',
    'save-settings', 'sync-templates', 'collect-garbage', 'global-rebuild',
    'nix-daemon-start', 'nix-daemon-stop', 'nix-daemon-restart', 'download-diagnostics',
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

// 3. Service actions and installation triggers delegated to helper
if (in_array($action, ['start', 'stop', 'restart', 'toggle-autostart', 'remove', 'install-cli', 'install-custom'])) {
    require_once __DIR__ . '/api_service.php';
    exit;
}
// Delegate all other system control actions to api_system.php
require_once __DIR__ . '/api_system.php';
