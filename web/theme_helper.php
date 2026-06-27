<?php
$nix_theme = 'black'; // default dark mode fallback
$dynamix_cfg_path = '/boot/config/plugins/dynamix/dynamix.cfg';
if (file_exists($dynamix_cfg_path)) {
    $dynamix_cfg = parse_ini_file($dynamix_cfg_path, true);
    if (isset($dynamix_cfg['display']['theme'])) {
        $nix_theme = strtolower(trim($dynamix_cfg['display']['theme']));
    }
}
?>
