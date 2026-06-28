<?php
/// Nix Plugin Output Streamer Initialization
///
/// Parses POST parameters and constructs helper command-lines.

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

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    die("This action requires POST.");
}
$csrf_token = $_POST['csrf_token'] ?? '';
$session_csrf = $_SESSION['csrf_token'] ?? '';
if (empty($session_csrf) || !hash_equals($session_csrf, $csrf_token)) {
    http_response_code(403);
    die("Invalid or missing CSRF token.");
}

set_time_limit(1800);
while (@ob_end_flush());
ob_implicit_flush(true);
header('Content-Type: text/html; charset=utf-8');
header('X-Accel-Buffering: no');
header("Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors 'none';");

$action = isset($_POST['action']) ? $_POST['action'] : '';
$uri = isset($_POST['uri']) ? $_POST['uri'] : (isset($_POST['package']) ? $_POST['package'] : '');
$type = isset($_POST['type']) ? $_POST['type'] : '';

$args = [];
$args[] = "--action " . escapeshellarg($action);
$args[] = "--uri " . escapeshellarg($uri);
if (!empty($type)) {
    $args[] = "--type " . escapeshellarg($type);
}

$forward_keys = [
    'appdata', 'media', 'puid', 'pgid', 'gpu', 'gpus',
    'extra_binds', 'port', 'bind_address',
    'network_isolation', 'command_override'
];

foreach ($forward_keys as $key) {
    if (isset($_POST[$key]) && $_POST[$key] !== '') {
        $flag = str_replace('_', '-', $key);
        $args[] = "--" . $flag . " " . escapeshellarg($_POST[$key]);
    }
}

if (isset($_POST['env_vars']) && $_POST['env_vars'] !== '') {
    $env_raw = $_POST['env_vars'];
    $env_decoded = json_decode($env_raw, true);
    if (json_last_error() !== JSON_ERROR_NONE || !is_array($env_decoded)) {
        http_response_code(400);
        die("Invalid env_vars JSON.");
    }
    foreach ($env_decoded as $env_key => $env_val) {
        if (!is_string($env_key)) {
            http_response_code(400);
            die("Invalid env_vars key.");
        }
        if (preg_match('/[\r\n]/', $env_key)) {
            http_response_code(400);
            die("Invalid env_vars key.");
        }
        if (stripos($env_key, 'LD_') === 0) {
            http_response_code(400);
            die("LD_PRELOAD injection rejected.");
        }
    }
    $args[] = "--env-vars " . escapeshellarg($env_raw);
}

if (isset($_POST['compile_locally']) && $_POST['compile_locally'] === '1') {
    $args[] = "--compile-locally";
}

$cmd = "/usr/local/emhttp/plugins/nix/nix-helper stream-install " . implode(" ", $args);
$title = "Installing Nix Resource: " . htmlspecialchars($uri);