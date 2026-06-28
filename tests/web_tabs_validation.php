<?php
// PHP Script to validate all Unraid Nix .page files for syntax and redeclaration errors.

$web_dir = dirname(__DIR__) . '/web';
$page_files = glob($web_dir . '/*.page');

if (empty($page_files)) {
    $web_dir = '/usr/local/emhttp/plugins/nix';
    $page_files = glob($web_dir . '/*.page');
}

if (empty($page_files)) {
    echo "No .page files found in local web/ or host plugin directory.\n";
    exit(1);
}

echo "Found " . count($page_files) . " page files to validate.\n";

// Concatenated PHP blocks
$concatenated_php = "<?php\n";
// Add stubs for Unraid global/theme helper variables and functions
$concatenated_php .= "
// MOCK UNRAID ENV
\$nix_theme = 'white';
\$is_nix_running = true;
\$store_path = '/mnt/user/system/nix';
\$autostart = 'yes';
\$show_in_nav = 'yes';
\$enable_sandbox = 'no';
\$allow_source_builds = 'no';
\$filter_presets_by_hardware = 'yes';
\$enable_pid_isolation = 'yes';
\$enable_uts_isolation = 'yes';
\$enable_ipc_isolation = 'yes';
\$auto_gc = 'no';
\$store_quota = '30';
\$build_cores = '0';
\$build_jobs = '0';
\$gc_min_free = '5';
\$gc_max_free = '10';
\$nix_channel = 'nixos-unstable';
\$settings_confirmed = 'yes';
\$has_kvm = true;
\$gpu_info = ['NVIDIA GeForce RTX 3080'];
\$num_flakes = 3;
\$suggested_quota = 24;

// Mock functions commonly used
if (!function_exists('parse_ini_file')) {
    function parse_ini_file(\$file) {
        return [];
    }
}
";

foreach ($page_files as $file) {
    echo "Processing " . basename($file) . "...\n";
    $content = file_get_contents($file);
    
    // Strip YAML frontmatter header (between ---)
    $parts = explode('---', $content, 2);
    $body = isset($parts[1]) ? $parts[1] : $content;
    
    // Extract all PHP blocks from the body
    preg_match_all('/<\?php(.*?)\?>/s', $body, $matches);
    foreach ($matches[1] as $php_block) {
        $concatenated_php .= $php_block . "\n";
    }
}

// Write the concatenated code to a temporary file
$temp_file = tempnam(sys_get_temp_dir(), 'nix_test_');
file_put_contents($temp_file, $concatenated_php);

// 1. Run syntax lint check (php -l)
$lint_output = [];
$lint_code = 0;
exec("php -l " . escapeshellarg($temp_file) . " 2>&1", $lint_output, $lint_code);

if ($lint_code !== 0) {
    echo "❌ Syntax validation FAILED:\n";
    echo implode("\n", $lint_output) . "\n";
    unlink($temp_file);
    exit(1);
}
echo "✓ Syntax lint check passed.\n";

// 2. Run execution check (php) to catch redeclaration errors
$eval_output = [];
$eval_code = 0;
exec("php " . escapeshellarg($temp_file) . " 2>&1", $eval_output, $eval_code);

unlink($temp_file);

if ($eval_code !== 0) {
    echo "❌ Execution / Redeclaration check FAILED:\n";
    echo implode("\n", $eval_output) . "\n";
    exit(1);
}

echo "✓ Execution / Redeclaration check passed successfully!\n";
echo "🎉 All WebUI page files are clean and validated!\n";
exit(0);
