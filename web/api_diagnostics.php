<?php
/// Nix Plugin WebGUI PHP API Diagnostics Packager
///
/// Dynamically compiles and downloads a tarball of logs and system specifications.

if ($action !== 'download-diagnostics') {
    return;
}

$tmp_dir = '/tmp/nix-diagnostics-' . uniqid();
if (!mkdir($tmp_dir, 0700, true)) {
    die("Failed to create temporary diagnostics directory.");
}

$cfg_dir = '/boot/config/plugins/nix';
if (file_exists("$cfg_dir/nix.cfg")) {
    copy("$cfg_dir/nix.cfg", "$tmp_dir/nix.cfg");
    @chmod("$tmp_dir/nix.cfg", 0600);
}
if (file_exists("$cfg_dir/process-compose.yml")) {
    copy("$cfg_dir/process-compose.yml", "$tmp_dir/process-compose.yml");
    @chmod("$tmp_dir/process-compose.yml", 0600);
}

$log_files = [
    '/var/log/nix-daemon.log' => 'nix-daemon.log',
    '/var/log/nix-plugin.log' => 'nix-plugin.log',
    '/var/log/nix-process-compose.log' => 'nix-process-compose.log'
];
foreach ($log_files as $src => $dest) {
    if (file_exists($src)) {
        copy($src, "$tmp_dir/$dest");
        @chmod("$tmp_dir/$dest", 0600);
    }
}

$services_log_dir = '/var/log/nix-services';
if (is_dir($services_log_dir)) {
    @mkdir("$tmp_dir/services", 0700);
    $files = glob("$services_log_dir/*.log");
    if ($files !== false) {
        foreach ($files as $file) {
            $dest = "$tmp_dir/services/" . basename($file);
            copy($file, $dest);
            @chmod($dest, 0600);
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
exec("/usr/local/emhttp/plugins/nix/nix-helper daemon-status 2>&1", $daemon_status);
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
@chmod("$tmp_dir/system_info.txt", 0600);

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
