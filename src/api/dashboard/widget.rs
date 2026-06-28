use super::rows::render_dashboard_rows;

/// Renders the complete native Unraid Dashboard widget tile structure (<table>).
pub fn render_dashboard_widget(api_port: u16) -> String {
    let rows_html = render_dashboard_rows(api_port);
    let mut html = String::new();

    html.push_str(r#"<table class="nix-dash-table" style="width: 100%; border-collapse: collapse; margin-top: 5px;">
        <thead>
            <tr style="border-bottom: 1px solid rgba(255, 255, 255, 0.08); text-align: left; font-size: 10px; color: #a0a0a5; text-transform: uppercase; letter-spacing: 0.5px;">
                <th style="padding: 6px 8px;">Service</th>
                <th style="padding: 6px 8px;">Status</th>
                <th style="padding: 6px 8px; text-align: center;">GPU</th>
                <th style="padding: 6px 8px; text-align: right;">Action</th>
            </tr>
        </thead>
        <tbody class="nix-dash-rows">"#);

    html.push_str(&rows_html);

    html.push_str(r#"</tbody>
    </table>
    <style>
    .nix-dash-table tr { background: transparent; transition: background 0.15s ease; }
    .nix-dash-table tr:hover { background: rgba(255, 255, 255, 0.015); }
    .nix-dash-toggle-btn:hover i.fa-play { color: #2ecc71 !important; text-shadow: 0 0 4px #2ecc71; }
    .nix-dash-toggle-btn:hover i.fa-stop { color: #e74c3c !important; text-shadow: 0 0 4px #e74c3c; }
    .nix-dash-gpu-active {
        animation: nix-gpu-pulse 1.6s infinite ease-in-out;
        color: #00a1ff !important;
        text-shadow: 0 0 6px #00a1ff, 0 0 12px rgba(0, 161, 255, 0.4);
        display: inline-block;
    }
    @keyframes nix-gpu-pulse {
        0% { transform: scale(1); opacity: 1; }
        50% { transform: scale(1.25); opacity: 0.65; color: #00e5ff !important; }
        100% { transform: scale(1); opacity: 1; }
    }
    </style>
    <script>
    if (typeof window.toggleDashService === 'undefined') {
        window.toggleDashService = function(name, action) {
            var btn = event.currentTarget;
            var icon = btn.querySelector('i');
            if (icon.classList.contains('fa-spinner')) return;

            icon.className = 'fa fa-spinner fa-spin';
            btn.disabled = true;

            var params = new URLSearchParams();
            params.append('service', name);

            fetch('/plugins/nix/api.php?action=' + action, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                body: params.toString()
            })
            .then(function(resp) { return resp.json(); })
            .then(function(data) {
                if (typeof window.refreshNixDash === 'function') {
                    window.refreshNixDash();
                }
            })
            .catch(function(err) {
                console.error('Error toggling service:', err);
                btn.disabled = false;
                icon.className = 'fa ' + (action === 'start' ? 'fa-play' : 'fa-stop');
            });
        };
    }

    if (typeof window.refreshNixDash === 'undefined') {
        window.nixDashVersion = window.nixDashVersion || 0;

        window.refreshNixDash = function() {
            var tbody = document.querySelector('tbody.nix-dash-rows');
            if (!tbody) {
                clearInterval(window.nixDashTimer);
                delete window.nixDashTimer;
                delete window.refreshNixDash;
                return;
            }

            fetch('/plugins/nix/api.php?action=get_dashboard_diff&since=' + (window.nixDashVersion || 0))
                .then(function(resp) { return resp.json(); })
                .then(function(data) {
                    if (!data || typeof data.version !== 'number') return;

                    if (Array.isArray(data.removed)) {
                        data.removed.forEach(function(name) {
                            var existing = tbody.querySelector('tr[data-service="' + name + '"]');
                            if (existing && existing.parentNode) {
                                existing.parentNode.removeChild(existing);
                            }
                        });
                    }

                    if (Array.isArray(data.changed)) {
                        data.changed.forEach(function(c) {
                            if (!c || !c.name || typeof c.html !== 'string') return;
                            var existing = tbody.querySelector('tr[data-service="' + c.name + '"]');
                            if (existing) {
                                existing.outerHTML = c.html;
                            } else {
                                tbody.insertAdjacentHTML('beforeend', c.html);
                            }
                        });
                    }

                    window.nixDashVersion = data.version;
                })
                .catch(function(err) { console.error('Error refreshing dashboard diff:', err); });
        };

        window.nixDashTimer = setInterval(window.refreshNixDash, 3000);
    }
    </script>"#);

    html
}
