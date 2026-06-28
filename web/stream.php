<?php
/// Nix Plugin Interactive Output Streamer
///
/// Executes Nix helper commands and streams output in real-time,
/// with live tailing support for service log files.

require_once "stream_init.php";
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
passthru($cmd . " 2>&1");
?>
<span class="cursor"></span></div></details>
    <div class="btn-container">
        <button id="logs-btn" class="logs-btn" style="display: none;">View Logs</button>
        <button id="close-btn" class="close-btn" onclick="window.close()" disabled>Close</button>
    </div>
</body>
</html>
