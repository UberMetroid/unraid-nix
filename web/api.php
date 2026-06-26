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
    
    $rendered = false;
    if (file_exists($log_file)) {
        $content = shell_exec("tail -n 200 " . escapeshellarg($log_file));
        if ($content !== null && trim($content) !== "") {
            $lines = explode("\n", trim($content));
            echo "<pre style='white-space: pre-wrap; word-wrap: break-word;'>";
            foreach ($lines as $line) {
                $line = trim($line);
                if (empty($line)) continue;
                $data = json_decode($line, true);
                if (is_array($data)) {
                    $time = isset($data['time']) ? $data['time'] : '';
                    $message = isset($data['message']) ? $data['message'] : '';
                    if (!empty($time)) {
                        $dt = date_create($time);
                        $time_str = $dt ? date_format($dt, 'Y-m-d H:i:s') : str_replace('T', ' ', substr($time, 0, 19));
                        echo "<span style='color:#888;'>[" . htmlspecialchars($time_str) . "]</span> " . htmlspecialchars($message) . "\n";
                    } else {
                        echo htmlspecialchars($message) . "\n";
                    }
                } else {
                    echo htmlspecialchars($line) . "\n";
                }
            }
            echo "</pre>";
            $rendered = true;
        }
    }
    
    if (!$rendered) {
        // Fallback to process-compose REST API
        $ch = curl_init("http://127.0.0.1:29704/process/logs/" . urlencode($service) . "/0/200");
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_TIMEOUT, 2);
        $response = curl_exec($ch);
        $http_code = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);
        
        if ($http_code === 200 && $response) {
            $data = json_decode($response, true);
            if (is_array($data) && isset($data['logs']) && is_array($data['logs'])) {
                $lines = $data['logs'];
                echo "<pre style='white-space: pre-wrap; word-wrap: break-word;'>";
                foreach ($lines as $line) {
                    echo htmlspecialchars($line) . "\n";
                }
                echo "</pre>";
            } else {
                echo "<p class='text-muted'>No logs found. If the service just started, it might take a few seconds to populate.</p>";
            }
        } else {
            echo "<p class='text-muted'>No logs found. If the service just started, it might take a few seconds to populate.</p>";
        }
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

// 2b. Render services table (outputs HTML directly, bypasses JSON header)
if ($action === 'render-services') {
    header('Content-Type: text/html');
    $html = shell_exec("/usr/local/emhttp/plugins/nix/nix-helper render services 2>&1");
    echo $html;
    exit;
}

// 2c. Get service metadata (JSON response)
if ($action === 'get-metadata') {
    $service = isset($_GET['service']) ? $_GET['service'] : '';
    if (empty($service) || preg_match('/[^a-zA-Z0-9_-]/', $service)) {
        error("Invalid or missing service name.");
    }
    echo json_encode(['success' => true, 'metadata' => get_service_metadata($service)]);
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
    @unlink("/boot/config/plugins/nix/metadata/" . $service . ".json");
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
        
        // 1. Create the primary Configuration Location (appdata) if it doesn't exist
        if (!empty($appdata)) {
            if (!file_exists($appdata)) {
                mkdir($appdata, 0777, true);
                @chown($appdata, intval($puid));
                @chgrp($appdata, intval($pgid));
            }
        }

        // 2. Create any additional host bind paths if they don't exist
        if (!empty($extra_binds)) {
            $binds_arr = json_decode($extra_binds, true);
            if (is_array($binds_arr)) {
                foreach ($binds_arr as $b) {
                    $host = trim($b['host']);
                    if (!empty($host)) {
                        if (!file_exists($host)) {
                            mkdir($host, 0777, true);
                            @chown($host, intval($puid));
                            @chgrp($host, intval($pgid));
                        }
                    }
                }
            }
        }

        // Parse name from URI (e.g. nixpkgs#radarr -> radarr)
        $name = str_replace('nixpkgs#', '', $uri);
        $name = last(explode('/', $name));
        $name = last(explode(':', $name));
        $name = last(explode('#', $name));
        $name = preg_replace('/[^a-zA-Z0-9_-]/', '', $name);

        // Format extra binds as host:sandbox comma-separated string
        $binds_arg = "";
        if (!empty($extra_binds)) {
            $binds_arr = json_decode($extra_binds, true);
            if (is_array($binds_arr)) {
                $parts = [];
                foreach ($binds_arr as $b) {
                    $host = trim($b['host']);
                    $sandbox = trim($b['sandbox']);
                    if (!empty($host) && !empty($sandbox)) {
                        $parts[] = $host . ":" . $sandbox;
                    }
                }
                $binds_arg = implode(",", $parts);
            }
        }

        // Fetch command string. Checks if it is a preset
        $cmd = "";
        if (in_array(strtolower($name), ['radarr', 'sonarr', 'jellyfin'])) {
            $cmd = shell_exec(format_preset_cmd($name, $appdata, $media, $puid, $pgid, $gpu, $binds_arg, $port, $bind_address));
        } else {
            // Build custom bubblewrap command
            $port_flag = empty($port) ? "" : " --port " . escapeshellarg($port);
            $addr_flag = empty($bind_address) ? "" : " --bind-address " . escapeshellarg($bind_address);
            $cmd = shell_exec("/usr/local/emhttp/plugins/nix/nix-helper sandbox --name " . escapeshellarg($name) . " --appdata " . escapeshellarg($appdata) . " --media " . escapeshellarg($media) . " --puid " . escapeshellarg($puid) . " --pgid " . escapeshellarg($pgid) . " --cmd " . escapeshellarg("nix run " . $uri) . " --extra-binds " . escapeshellarg($binds_arg) . $port_flag . $addr_flag);
        }


        $output = [];
        $code = 0;
        exec("/usr/local/emhttp/plugins/nix/nix-helper add-service " . escapeshellarg($name) . " " . escapeshellarg(trim($cmd)) . " 2>&1", $output, $code);
        if ($code !== 0) {
            error(implode("\n", $output));
        }

        // Save service metadata for editing
        $metadata = [
            'name' => $name,
            'uri' => $uri,
            'appdata' => $appdata,
            'puid' => $puid,
            'pgid' => $pgid,
            'gpu' => $gpu,
            'extra_binds' => $extra_binds,
            'port' => $port,
            'bind_address' => $bind_address
        ];
        $meta_dir = "/boot/config/plugins/nix/metadata";
        if (!file_exists($meta_dir)) {
            mkdir($meta_dir, 0777, true);
        }
        file_put_contents($meta_dir . "/" . $name . ".json", json_encode($metadata));

        // Restart supervisor to apply the new service definition and launch it
        restart_nix_supervisor();

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
    
    // Read old config to check if the store path has changed
    $old_store_path = '';
    $cfg_file = "/boot/config/plugins/nix/nix.cfg";
    if (file_exists($cfg_file)) {
        $old_cfg = parse_ini_file($cfg_file);
        if (isset($old_cfg['NIX_STORE_PATH'])) {
            $old_store_path = $old_cfg['NIX_STORE_PATH'];
        }
    }
    if (empty($old_store_path)) {
        $old_store_path = detect_default_store_path(); // default
    }

    $store_path = rtrim($store_path, '/');
    $old_store_path = rtrim($old_store_path, '/');

    // If path changed and the old path is populated, migrate the data!
    $migration_performed = false;
    if (!empty($store_path) && $store_path !== $old_store_path) {
        // If old store exists and contains files, we should migrate
        if (file_exists($old_store_path) && count(glob("$old_store_path/*")) > 0) {
            // 1. Stop services and Nix daemon
            shell_exec("/usr/local/emhttp/plugins/nix/event/stopping_svcs >/dev/null 2>&1");
            
            // 2. Double check /nix is unmounted
            shell_exec("umount -l /nix >/dev/null 2>&1");
            
            // 3. Create new directory
            if (!file_exists($store_path)) {
                mkdir($store_path, 0777, true);
            }
            
            // 4. Sync files to the new location
            shell_exec("rsync -aHAX " . escapeshellarg($old_store_path . "/") . " " . escapeshellarg($store_path . "/") . " >/dev/null 2>&1");
            
            // Mark migration performed so we restart daemon and services
            $migration_performed = true;
        }
    }

    $cfg_dir = "/boot/config/plugins/nix";
    if (!file_exists($cfg_dir)) {
        mkdir($cfg_dir, 0777, true);
    }
    
    $cfg_content = "NIX_STORE_PATH=\"" . addslashes($store_path) . "\"\n";
    $cfg_content .= "AUTOSTART_FLAKES=\"" . addslashes($autostart) . "\"\n";
    $cfg_content .= "ENABLE_STORAGE_SANDBOX=\"" . addslashes($enable_sandbox) . "\"\n";
    $cfg_content .= "ENABLE_CLI_INSTALL=\"" . addslashes($enable_cli) . "\"\n";
    $cfg_content .= "SHOW_IN_NAVIGATION=\"" . addslashes($show_in_nav) . "\"\n";
    
    if (file_put_contents($cfg_dir . "/nix.cfg", $cfg_content) === false) {
        error("Failed to write nix.cfg to flash drive.");
    }

    // Dynamically update active Nix.page menu registry
    $nix_page_file = "/usr/local/emhttp/plugins/nix/Nix.page";
    $nix_launcher_file = "/usr/local/emhttp/plugins/nix/NixLauncher.page";
    if (file_exists($nix_page_file)) {
        $nix_page_content = file_get_contents($nix_page_file);
        if ($show_in_nav === 'yes') {
            $nix_page_content = preg_replace('/^Menu="[^"]*"/m', 'Menu="Tasks:95"', $nix_page_content);
            $launcher_content = "Menu=\"Utilities\"\nTitle=\"Nix\"\nIcon=\"nix.png\"\n---\n<script>window.location.href = '/Settings/Nix';</script>\n";
            file_put_contents($nix_launcher_file, $launcher_content);
        } else {
            $nix_page_content = preg_replace('/^Menu="[^"]*"/m', 'Menu="Utilities"', $nix_page_content);
            if (file_exists($nix_launcher_file)) {
                unlink($nix_launcher_file);
            }
        }
        file_put_contents($nix_page_file, $nix_page_content);
    }

    if ($migration_performed) {
        // Start services back up using the updated config
        shell_exec("/usr/local/emhttp/plugins/nix/event/disks_mounted >/dev/null 2>&1");
    }
    
    success();
}

// Helper methods for string parsing in PHP
function last($arr) {
    return end($arr);
}

function format_preset_cmd($name, $appdata, $media, $puid, $pgid, $gpu, $extra_binds = '', $port = '', $bind_address = '') {
    $media_arg = empty($media) ? "-" : $media;
    $binds_arg = empty($extra_binds) ? "-" : $extra_binds;
    $port_arg = empty($port) ? "-" : $port;
    $addr_arg = empty($bind_address) ? "-" : $bind_address;
    return "/usr/local/emhttp/plugins/nix/nix-helper preset " . escapeshellarg($name) . " " . escapeshellarg($appdata) . " " . escapeshellarg($media_arg) . " " . escapeshellarg($puid) . " " . escapeshellarg($pgid) . " " . escapeshellarg($gpu) . " " . escapeshellarg($binds_arg) . " " . escapeshellarg($port_arg) . " " . escapeshellarg($addr_arg);
}

function restart_nix_supervisor() {
    $pid_file = "/var/run/nix-process-compose.pid";
    $cfg_file = "/boot/config/plugins/nix/process-compose.yml";
    
    // 1. Gracefully shut down process-compose via port 29704
    shell_exec(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nix run nixpkgs#process-compose -- -p 29704 down >/dev/null 2>&1");
    
    // Wait up to 3 seconds for the port to be freed
    $freed = false;
    for ($i = 0; $i < 30; $i++) {
        $port_status = [];
        $port_code = 0;
        exec("fuser 29704/tcp 2>/dev/null", $port_status, $port_code);
        if ($port_code !== 0) {
            $freed = true;
            break;
        }
        usleep(100000);
    }
    
    // If still running, force kill any process using port 29704
    if (!$freed) {
        $port_status = [];
        $port_code = 0;
        exec("fuser 29704/tcp 2>/dev/null", $port_status, $port_code);
        if ($port_code === 0 && !empty($port_status)) {
            $pids = preg_split('/\s+/', trim(implode(' ', $port_status)));
            foreach ($pids as $p) {
                if (is_numeric($p)) {
                    exec("kill -9 " . escapeshellarg($p) . " >/dev/null 2>&1");
                }
            }
        }
    }
    @unlink($pid_file);
    
    // 2. Start it up
    if (file_exists($cfg_file)) {
        shell_exec("mkdir -p /var/log/nix-services");
        $cmd = ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh && nohup nix run nixpkgs#process-compose -- -p 29704 -f " . escapeshellarg($cfg_file) . " --tui=false --keep-project > /var/log/nix-process-compose.log 2>&1 & echo \$! > " . escapeshellarg($pid_file);
        shell_exec($cmd);
    }
}

function detect_default_store_path() {
    $cfg_file = '/boot/config/shares/system.cfg';
    if (file_exists($cfg_file)) {
        $share_cfg = parse_ini_file($cfg_file);
        if ($share_cfg !== false) {
            $pool = isset($share_cfg['shareCachePool']) ? trim($share_cfg['shareCachePool'], '"') : '';
            if (empty($pool) && isset($share_cfg['shareUseCache'])) {
                $use_cache = trim($share_cfg['shareUseCache'], '"');
                if (in_array($use_cache, ['yes', 'prefer', 'only'])) {
                    $pool = 'cache';
                }
            }
            if (!empty($pool) && is_dir("/mnt/" . $pool . "/system")) {
                return "/mnt/" . $pool . "/system/nix";
            }
        }
    }
    
    if (is_dir('/mnt/user/system')) {
        return '/mnt/user/system/nix';
    }

    return '';
}

function detect_appdata_root() {
    $cfg_file = '/boot/config/shares/appdata.cfg';
    if (file_exists($cfg_file)) {
        $share_cfg = parse_ini_file($cfg_file);
        if ($share_cfg !== false) {
            $pool = isset($share_cfg['shareCachePool']) ? trim($share_cfg['shareCachePool'], '"') : '';
            if (empty($pool) && isset($share_cfg['shareUseCache'])) {
                $use_cache = trim($share_cfg['shareUseCache'], '"');
                if (in_array($use_cache, ['yes', 'prefer', 'only'])) {
                    $pool = 'cache';
                }
            }
            if (!empty($pool) && is_dir("/mnt/" . $pool . "/appdata")) {
                return "/mnt/" . $pool . "/appdata";
            }
        }
    }
    
    if (is_dir('/mnt/user/appdata')) {
        return '/mnt/user/appdata';
    }

    return '';
}

function get_service_metadata($name) {
    $meta_file = "/boot/config/plugins/nix/metadata/" . $name . ".json";
    if (file_exists($meta_file)) {
        $data = json_decode(file_get_contents($meta_file), true);
        if (is_array($data)) {
            return $data;
        }
    }
    
    $detected_root = detect_appdata_root();
    $fallback_appdata_root = !empty($detected_root) ? $detected_root : '/mnt/user/appdata';

    // Fallback: Parse from process-compose.yml
    $cfg_file = "/boot/config/plugins/nix/process-compose.yml";
    if (file_exists($cfg_file)) {
        $content = file_get_contents($cfg_file);
        $pattern = '/\s+' . preg_quote($name, '/') . ':\s*\n(\s+command:\s*(.*?)\n)?/s';
        if (preg_match($pattern, $content, $matches)) {
            $lines = explode("\n", $matches[0]);
            $cmd = "";
            foreach ($lines as $line) {
                if (preg_match('/^\s+command:\s*(.*)/', $line, $cmd_match)) {
                    $cmd = trim($cmd_match[1], " '\"");
                    break;
                }
            }
            
            if (!empty($cmd)) {
                $puid = '99';
                $pgid = '100';
                $appdata = $fallback_appdata_root . '/' . $name;
                $uri = 'nixpkgs#' . $name;
                $gpu = '0';
                
                if (preg_match('/--reuid=(\d+)/', $cmd, $m)) {
                    $puid = $m[1];
                }
                if (preg_match('/--regid=(\d+)/', $cmd, $m)) {
                    $pgid = $m[1];
                }
                if (preg_match('/export HOME=([^\s&;"]+)/', $cmd, $m)) {
                    $appdata = $m[1];
                }
                if (preg_match('/exec (nix run |nixpkgs#)([^\s"]+)/', $cmd, $m)) {
                    if (strpos($m[2], '#') !== false || strpos($m[2], ':') !== false) {
                        $uri = $m[2];
                    } else {
                        $uri = 'nixpkgs#' . $m[2];
                    }
                }
                
                return [
                    'name' => $name,
                    'uri' => $uri,
                    'appdata' => $appdata,
                    'puid' => $puid,
                    'pgid' => $pgid,
                    'gpu' => $gpu,
                    'extra_binds' => '[]'
                ];
            }
        }
    }
    
    return [
        'name' => $name,
        'uri' => 'nixpkgs#' . $name,
        'appdata' => $fallback_appdata_root . '/' . $name,
        'puid' => '99',
        'pgid' => '100',
        'gpu' => '0',
        'extra_binds' => '[]'
    ];
}

error("Unknown API action.");
