function setStepStatus(stepNum, status, text, badgeText) {
    var step = document.getElementById("step-" + stepNum);
    if (!step) return;
    var iconClass = "fa-circle";
    var iconColor = "#444";
    var opacity = "0.5";
    if (status === "running") {
        iconClass = "fa-circle-o-notch fa-spin";
        iconColor = "#00a1ff";
        opacity = "1.0";
    } else if (status === "done") {
        iconClass = "fa-check-circle";
        iconColor = "#2ecc71";
        opacity = "1.0";
    } else if (status === "failed") {
        iconClass = "fa-times-circle";
        iconColor = "#e74c3c";
        opacity = "1.0";
    }
    step.style.opacity = opacity;
    var span = step.querySelector("span");
    if (span) {
        span.style.color = (status === "pending") ? "#aaa" : "#eee";
        span.innerHTML = "<i class='fa " + iconClass + "' style='margin-right: 8px; color: " + iconColor + ";'></i> " + text;
    }
    var badge = step.querySelector(".step-badge");
    if (badge) {
        badge.innerText = badgeText;
        if (status === "running") {
            badge.style.background = "rgba(0, 161, 255, 0.1)";
            badge.style.color = "#00a1ff";
        } else if (status === "done") {
            badge.style.background = "rgba(46, 204, 113, 0.1)";
            badge.style.color = "#2ecc71";
        } else if (status === "failed") {
            badge.style.background = "rgba(231, 76, 60, 0.1)";
            badge.style.color = "#e74c3c";
        } else {
            badge.style.background = "#232326";
            badge.style.color = "#888";
        }
    }
}

function openServiceLogs(svc) {
    if (!svc) return;
    var w = 800, h = 600;
    var left = (screen.width/2)-(w/2);
    var top = (screen.height/2)-(h/2);
    window.open('/plugins/nix/api.php?action=logs&service=' + svc, 'Service Logs: ' + svc, 'toolbar=no, location=no, directories=no, status=no, menubar=no, scrollbars=yes, resizable=yes, copyhistory=no, width='+w+', height='+h+', top='+top+', left='+left);
}

function finishInstallation(code, action, type, svc, reportHtml) {
    if (window.scrollInterval) clearInterval(window.scrollInterval);
    var container = document.getElementById("output-container");
    var btn = document.getElementById("close-btn");
    var spinner = document.getElementById("status-spinner");
    if (container) {
        container.className = "";
        var cursor = container.querySelector(".cursor");
        if (cursor) cursor.remove();
        
        var span = document.createElement("span");
        if (code !== 0) {
            span.className = "error";
            span.innerText = "\n\n[FATAL] Installation failed with exit status " + code;
            if (spinner) { spinner.className = ""; spinner.innerHTML = "<i class='fa fa-times-circle error' style='font-size:18px;'></i>"; }
            container.appendChild(span);
            
            var rawConsole = document.getElementById("raw-console");
            if (rawConsole) { rawConsole.open = true; }
            
            setStepStatus(1, 'failed', '1. Resolving Flake package & dependencies...', 'Failed');
            if (document.getElementById('overall-status')) {
                document.getElementById('overall-status').innerHTML = '<i class="fa fa-times-circle error"></i> Failed';
            }
        } else {
            span.className = "success";
            span.innerText = "\n\n[SUCCESS] Installation finished successfully!";
            if (spinner) { spinner.className = ""; spinner.innerHTML = "<i class='fa fa-check-circle success' style='font-size:18px;'></i>"; }
            container.appendChild(span);
            
            setStepStatus(1, 'done', '1. Resolving Flake package & dependencies...', 'Complete');
            if (action === 'install-custom' && type === 'service') {
                setStepStatus(2, 'done', '2. Running pre-flight checks (ports & permissions)...', 'Complete');
                setStepStatus(3, 'done', '3. Constructing sandbox jail & mounting paths...', 'Complete');
                setStepStatus(4, 'done', '4. Injecting env variables & log rotation limits...', 'Complete');
                setStepStatus(5, 'done', '5. Starting process supervisor & verifying liveness...', 'Complete');
            }
            if (document.getElementById('overall-status')) {
                document.getElementById('overall-status').innerHTML = '<i class="fa fa-check-circle success"></i> Complete';
            }
            
            if (action === 'install-custom' && type === 'service' && svc) {
                var logsBtn = document.getElementById("logs-btn");
                if (logsBtn) { logsBtn.style.display = "inline-block"; logsBtn.className = "logs-btn enabled"; }
                
                var dashboard = document.getElementById("status-dashboard");
                if (dashboard) {
                    var reportDiv = document.createElement("div");
                    reportDiv.innerHTML = reportHtml;
                    dashboard.appendChild(reportDiv);
                }
            }
        }
        container.scrollTop = container.scrollHeight;
    }
    if (btn) {
        btn.disabled = false;
        btn.className = "close-btn enabled";
    }
}
