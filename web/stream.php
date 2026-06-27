<?php
/// Nix Plugin Interactive Output Streamer
///
/// Executes Nix helper commands and streams output in real-time,
/// with live tailing support for service log files.

set_time_limit(120);
while (@ob_end_flush());
ob_implicit_flush(true);
header('Content-Type: text/html; charset=utf-8');
header('X-Accel-Buffering: no');

$action = isset($_POST['action']) ? $_POST['action'] : '';
$cmd = ''; $title = ''; $uri = '';

if ($action === 'install-cli') {
    $package = isset($_POST['package']) ? $_POST['package'] : '';
    if (empty($package)) { echo "Missing package name."; exit; }
    $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($package);
    $title = "Installing CLI Package: " . htmlspecialchars($package);
} elseif ($action === 'install-custom') {
    $uri = isset($_POST['uri']) ? $_POST['uri'] : '';
    $type = isset($_POST['type']) ? $_POST['type'] : '';
    if (empty($uri)) { echo "Missing Flake URI."; exit; }
    if ($type === 'cli') {
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install " . escapeshellarg($uri);
        $title = "Installing CLI Tool: " . htmlspecialchars($uri);
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
        $env_vars = isset($_POST['env_vars']) ? $_POST['env_vars'] : '';
        $compile_locally = isset($_POST['compile_locally']) ? $_POST['compile_locally'] : '0';
        $cmd = "/usr/local/emhttp/plugins/nix/nix-helper install-service " .
               "--uri " . escapeshellarg($uri) . " --appdata " . escapeshellarg($appdata) . " " .
               "--media " . escapeshellarg($media) . " --puid " . escapeshellarg($puid) . " " .
               "--pgid " . escapeshellarg($pgid) . " --gpu " . escapeshellarg($gpu) . " " .
               "--gpus " . escapeshellarg($gpus) . " " .
               "--extra-binds " . escapeshellarg($extra_binds) . " --port " . escapeshellarg($port) . " " .
               "--bind-address " . escapeshellarg($bind_address) . " --env-vars " . escapeshellarg($env_vars);
        if ($compile_locally === '1') {
            $cmd .= " --compile-locally";
        }
        $title = "Installing Nix Service: " . htmlspecialchars($uri);
    } else { echo "Invalid installation type."; exit; }
} else { echo "Unknown action."; exit; }
?>
<!DOCTYPE html>
<html>
<head>
    <title>Nix Installation Console</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css">
    <link rel="stylesheet" href="/plugins/nix/styles_widgets.css?v=<?php echo time(); ?>">
    <script src="/plugins/nix/stream.js?v=<?php echo time(); ?>"></script>
</head>
<body class="nix-console-body">
    <h3 style="margin-top: 0; margin-bottom: 10px; color: #fff; display: flex; align-items: center; justify-content: space-between;">
        <span><i class="fa fa-cogs"></i> Nix Flake Installer</span>
        <div id="status-spinner" class="spinner-ring"></div>
    </h3>
    <div style="font-size: 12px; color: #888; margin-bottom: 15px;"><?php echo htmlspecialchars($title); ?></div>
    
    <div id="status-dashboard">
        <div class="progress-container" style="padding: 15px; background: rgba(0, 161, 255, 0.02); border: 1px solid #2d2d30; border-radius: 6px; margin-bottom: 15px; font-family: 'Clear Sans', sans-serif;">
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 15px;">
                <span style="font-weight: 600; color: #fff;">Installation Progress</span>
                <span id="overall-status" style="font-size: 12px; color: #00a1ff; display: flex; align-items: center; gap: 6px; font-weight: 600;">
                    <i class="fa fa-circle-o-notch fa-spin"></i> In Progress
                </span>
            </div>
            <div style="display: flex; flex-direction: column; gap: 12px;">
                <div id="step-1" style="display: flex; align-items: center; justify-content: space-between; font-size: 12.5px;">
                    <span style="color: #eee;"><i class="fa fa-circle-o-notch fa-spin" style="margin-right: 8px; color: #00a1ff;"></i> 1. Resolving Flake package & dependencies...</span>
                    <span class="step-badge" style="background: rgba(0, 161, 255, 0.1); color: #00a1ff; padding: 2px 8px; border-radius: 10px; font-size: 10.5px; font-weight: 600;">Active</span>
                </div>
                <?php if ($action === 'install-custom' && $type === 'service'): ?>
                <div id="step-2" style="display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; opacity: 0.5;">
                    <span style="color: #aaa;"><i class="fa fa-circle" style="margin-right: 8px; color: #444;"></i> 2. Running pre-flight checks (ports & permissions)...</span>
                    <span class="step-badge" style="background: #232326; color: #888; padding: 2px 8px; border-radius: 10px; font-size: 10.5px; font-weight: 600;">Pending</span>
                </div>
                <div id="step-3" style="display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; opacity: 0.5;">
                    <span style="color: #aaa;"><i class="fa fa-circle" style="margin-right: 8px; color: #444;"></i> 3. Constructing sandbox jail & mounting paths...</span>
                    <span class="step-badge" style="background: #232326; color: #888; padding: 2px 8px; border-radius: 10px; font-size: 10.5px; font-weight: 600;">Pending</span>
                </div>
                <div id="step-4" style="display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; opacity: 0.5;">
                    <span style="color: #aaa;"><i class="fa fa-circle" style="margin-right: 8px; color: #444;"></i> 4. Injecting env variables & log rotation limits...</span>
                    <span class="step-badge" style="background: #232326; color: #888; padding: 2px 8px; border-radius: 10px; font-size: 10.5px; font-weight: 600;">Pending</span>
                </div>
                <div id="step-5" style="display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; opacity: 0.5;">
                    <span style="color: #aaa;"><i class="fa fa-circle" style="margin-right: 8px; color: #444;"></i> 5. Starting process supervisor & verifying liveness...</span>
                    <span class="step-badge" style="background: #232326; color: #888; padding: 2px 8px; border-radius: 10px; font-size: 10.5px; font-weight: 600;">Pending</span>
                </div>
                <?php endif; ?>
            </div>
        </div>
    </div>

    <details id="raw-console" style="margin-bottom: 15px; border: 1px solid #2d2d30; border-radius: 6px; background: #09090a; padding: 5px 12px;">
        <summary style="cursor: pointer; color: #888; font-weight: 600; font-size: 11px; outline: none; padding: 8px 0; user-select: none;">Show Raw Console Logs</summary>
        <div id="output-container" class="running" style="margin-top: 5px; margin-bottom: 10px; border: none; background: #070708; max-height: 180px; overflow-y: auto; font-family: 'Courier New', Courier, monospace; font-size: 11px; color: #888; padding: 10px; border-radius: 4px;">
    <script>
        var container = document.getElementById("output-container");
        window.scrollInterval = setInterval(function() { if (container) { container.scrollTop = container.scrollHeight; } }, 50);
    </script>
<span class="ascii-art">
<?php
echo "  _   _ _____  __\n";
echo " | \\ | |_ _\\ \\/ /\n";
echo " |  \\| || | \\  /\n";
echo " | |\\  || | /  \\\n";
echo " |_| \\__|___/_/\\_\\\n";
?>
</span>
==================================================
Starting Nix Installation Process...
==================================================

<?php
flush();

$descriptor = [0 => ["pipe", "r"], 1 => ["pipe", "w"], 2 => ["pipe", "w"]];
$env = array_merge($_ENV, [
    'NIX_REMOTE' => 'daemon',
    'PATH' => '/nix/var/nix/profiles/default/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin'
]);
$proc = proc_open($cmd . " 2>&1", $descriptor, $pipes, null, $env);

if (is_resource($proc)) {
    fclose($pipes[0]);
    while (!feof($pipes[1])) {
        $line = fgets($pipes[1]);
        if ($line !== false) { echo htmlspecialchars($line); flush(); }
    }
    fclose($pipes[1]); fclose($pipes[2]);
    $code = proc_close($proc);
} else {
    $code = -1;
    echo "<span class='error'>Failed to execute installation process.</span>\n";
}

if ($code === 0) {
    if ($action === 'install-custom' && $type === 'service') {
        echo "<script>if (typeof setStepStatus === 'function') { setStepStatus(1, 'done', '1. Resolving Flake package & dependencies...', 'Complete'); setStepStatus(2, 'running', '2. Running pre-flight checks (ports & permissions)...', 'Running'); }</script>";
        flush(); usleep(400000);
        echo "<script>if (typeof setStepStatus === 'function') { setStepStatus(2, 'done', '2. Running pre-flight checks (ports & permissions)...', 'Complete'); setStepStatus(3, 'running', '3. Constructing sandbox jail & mounting paths...', 'Running'); }</script>";
        flush(); usleep(400000);
        echo "<script>if (typeof setStepStatus === 'function') { setStepStatus(3, 'done', '3. Constructing sandbox jail & mounting paths...', 'Complete'); setStepStatus(4, 'running', '4. Injecting env variables & log rotation limits...', 'Running'); }</script>";
        flush(); usleep(400000);
        echo "<script>if (typeof setStepStatus === 'function') { setStepStatus(4, 'done', '4. Injecting env variables & log rotation limits...', 'Complete'); setStepStatus(5, 'running', '5. Starting process supervisor & verifying liveness...', 'Running'); }</script>";
    } else {
        echo "<script>if (typeof setStepStatus === 'function') { setStepStatus(1, 'done', '1. Resolving Flake package & dependencies...', 'Complete'); }</script>";
    }
    flush();
} else {
    echo "<script>if (typeof setStepStatus === 'function') { setStepStatus(1, 'failed', '1. Resolving Flake package & dependencies...', 'Failed'); if (document.getElementById('overall-status')) { document.getElementById('overall-status').innerHTML = '<i class=\"fa fa-times-circle error\"></i> Failed'; } }</script>";
    flush();
}

// Live tailing logs for newly installed services
if ($code === 0 && $action === 'install-custom' && $type === 'service') {
    $svc = str_replace('nixpkgs#', '', strtolower($uri));
    $parts1 = explode('/', $svc); $svc = end($parts1);
    $parts2 = explode(':', $svc); $svc = end($parts2);
    $parts3 = explode('#', $svc); $svc = end($parts3);
    $svc = preg_replace('/[^a-zA-Z0-9_-]/', '', $svc);
    $log_file = "/var/log/nix-services/{$svc}.log";
    echo "\nService config written. Waiting for service to spawn logs...\n"; flush();
    
    $opened = false; $start = time(); $handle = null;
    while (time() - $start < 8) {
        if (file_exists($log_file)) {
            $handle = fopen($log_file, 'r');
            if ($handle) { $opened = true; break; }
        }
        usleep(500000);
    }
    
    if ($opened && $handle) {
        echo "Tailing startup logs for service: {$svc}...\n";
        echo "--------------------------------------------------\n"; flush();
        $last_pos = 0; $tail_start = time(); $success_found = false;
        while (time() - $tail_start < 45) {
            clearstatcache(true, $log_file);
            $len = filesize($log_file);
            if ($len > $last_pos) {
                fseek($handle, $last_pos);
                while (($line = fgets($handle)) !== false) {
                    $log_data = json_decode($line, true);
                    if (is_array($log_data) && isset($log_data['message'])) {
                        echo htmlspecialchars($log_data['message']) . "\n";
                    } else { echo htmlspecialchars($line); }
                    flush();
                }
                $last_pos = ftell($handle);
            }
            
            $ctx = stream_context_create(['http' => ['timeout' => 1.0]]);
            $status_json = @file_get_contents("http://127.0.0.1:29704/processes", false, $ctx);
            if ($status_json) {
                $status_data = json_decode($status_json, true);
                if (is_array($status_data) && isset($status_data['data'])) {
                    foreach ($status_data['data'] as $proc_status) {
                        if ($proc_status['name'] === $svc) {
                            $state = strtolower($proc_status['status']);
                            if ($state === 'running') {
                                echo "\n[SUCCESS] Service is now running!\n";
                                exec("/usr/local/emhttp/plugins/nix/nix-helper autostart " . escapeshellarg($svc) . " on 2>&1");
                                $success_found = true;
                                break 2;
                            }
                            if ($state === 'failed') {
                                echo "\n[FATAL] Service failed to start!\n";
                                $code = -1;
                                exec("/usr/local/emhttp/plugins/nix/nix-helper autostart " . escapeshellarg($svc) . " off 2>&1");
                                $success_found = false;
                                break 2;
                            }
                        }
                    }
                }
            }
            usleep(500000);
        }
        fclose($handle);
        
        if (!$success_found && $code === 0) {
            echo "\n[WARNING] Service startup verification timed out. Turning off autostart.\n";
            $code = -1;
            exec("/usr/local/emhttp/plugins/nix/nix-helper autostart " . escapeshellarg($svc) . " off 2>&1");
        }
    } else {
        echo "No logs spawned within 8 seconds. Service might be starting slowly in the background. Turning off autostart.\n";
        $code = -1;
        exec("/usr/local/emhttp/plugins/nix/nix-helper autostart " . escapeshellarg($svc) . " off 2>&1");
    }
}

// Load metadata and build validation report HTML if service installation succeeded
$report_html = '';
if ($code === 0 && $action === 'install-custom' && $type === 'service' && isset($svc)) {
    $report_html = shell_exec("/usr/local/emhttp/plugins/nix/nix-helper render report " . escapeshellarg($svc) . " 2>&1");
}
?>
<span class="cursor"></span></div></details>
    <div class="btn-container">
        <button id="logs-btn" class="logs-btn" onclick="openServiceLogs('<?php echo isset($svc) ? $svc : ''; ?>')">View Logs</button>
        <button id="close-btn" class="close-btn" onclick="window.close()" disabled>Close</button>
    </div>
    <script>
        finishInstallation(
            <?php echo $code; ?>,
            <?php echo json_encode($action); ?>,
            <?php echo json_encode($type); ?>,
            <?php echo json_encode(isset($svc) ? $svc : null); ?>,
            <?php echo json_encode($report_html); ?>
        );
    </script>
</body>
</html>
