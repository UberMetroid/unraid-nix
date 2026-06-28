use crate::api::utils::{html_escape, js_escape};

pub struct RowTemplateData<'a> {
    pub name: &'a str,
    pub bg: &'a str,
    pub border: &'a str,
    pub color: &'a str,
    pub icon: &'a str,
    pub version_badge: &'a str,
    pub time_html: &'a str,
    pub status_class: &'a str,
    pub status_label: &'a str,
    pub resources_html: &'a str,
    pub lan_ip_port_html: &'a str,
    pub mapped_drives_html: &'a str,
    pub ports_html: &'a str,
    pub rollback_html: &'a str,
    pub start_btn: &'a str,
    pub stop_btn: &'a str,
    pub edit_btn: &'a str,
    pub logs_btn: &'a str,
    pub autostart_html: &'a str,
}

pub fn build_row_html(d: &RowTemplateData) -> String {
    let name_html = html_escape(d.name);
    let name_js = html_escape(&js_escape(d.name));
    let icon_html = html_escape(d.icon);
    format!(
        r#"<div class="nix-preset-card nix-service-card" data-name="{}" style="background: var(--nix-bg-secondary); border: 1px solid var(--nix-border-primary); border-radius: 6px; padding: 16px; display: flex; flex-direction: column; justify-content: space-between; transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease; min-height: 350px; height: auto; position: relative;">
            <div>
                <!-- Top Row: Icon, Name + Path/Version on Left, Uptime & Status Dot on Right -->
                <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; margin-bottom: 10px;">
                    <div style="display: flex; align-items: flex-start; gap: 10px; min-width: 0; flex: 1;">
                        <div style="width: 32px; height: 32px; border-radius: 4px; background: {}; border: 1px solid {}; display: flex; align-items: center; justify-content: center; color: {}; flex-shrink: 0; margin-top: 2px;">
                            <i class="fa {}" style="font-size: 15px;"></i>
                        </div>
                        <div style="display: flex; flex-direction: column; overflow: hidden; min-width: 0; flex: 1;">
                            <strong style="font-size: 14px; color: var(--nix-text-primary); word-break: break-word; overflow-wrap: break-word;" title="{}">{}</strong>
                            <span style="font-family: monospace; color: var(--nix-text-secondary); font-size: 10px; margin-top: 2px; display: inline-flex; align-items: center; gap: 6px; flex-wrap: wrap;">
                                <span>nixpkgs#{}</span>
                                {}
                            </span>
                        </div>
                    </div>
                    <div style="display: flex; align-items: center; gap: 6px; flex-shrink: 0; margin-top: 6px;">
                        {}
                        <span class="status-dot {}" data-service="{}" title="{}" style="margin-top: 0;"></span>
                    </div>
                </div>

                <!-- Info list -->
                <div style="display: flex; flex-direction: column; gap: 8px; font-size: 11px; border-top: 1px solid var(--nix-border-primary); padding-top: 10px;">
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">ACTIVITY</span>
                        <div style="padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">CONNECTION</span>
                        <div style="padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">MOUNTS</span>
                        <div style="display: flex; flex-direction: column; gap: 3px; padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">PORTS</span>
                        <div style="display: flex; flex-direction: column; gap: 3px; padding-left: 6px;">{}</div>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 3px; line-height: 1.3;">
                        <span style="color: var(--nix-text-secondary); font-size: 10px; font-weight: 600;">ROLLBACK</span>
                        <div style="padding-left: 6px;">{}</div>
                    </div>
                </div>
            </div>

            <!-- Bottom Row: Controls Toolbar -->
            <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--nix-border-primary); padding-top: 10px; margin-top: 12px;">
                <div style="display: flex; gap: 6px; align-items: center;">
                    {}
                    {}
                    {}
                    {}
                    {}
                </div>
                <button type="button" class="nix-btn nix-btn-sm" style="color: #e74c3c; border-color: var(--nix-border-primary); margin: 0; display: inline-flex; align-items: center; justify-content: center; height: 32px; width: 32px;" onclick="removeService('{}')" title="Remove"><i class="fa fa-trash-o" style="color: #e74c3c;"></i></button>
            </div>
        </div>"#,
        name_html,
        d.bg,
        d.border,
        d.color,
        icon_html,
        name_html,
        name_html,
        name_html,
        d.version_badge,
        d.time_html,
        d.status_class,
        name_html,
        d.status_label,
        d.resources_html,
        d.lan_ip_port_html,
        d.mapped_drives_html,
        d.ports_html,
        d.rollback_html,
        d.start_btn,
        d.stop_btn,
        d.edit_btn,
        d.logs_btn,
        d.autostart_html,
        name_js
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data() -> RowTemplateData<'static> {
        RowTemplateData {
            name: "my-service",
            bg: "#000",
            border: "#111",
            color: "#fff",
            icon: "fa-server",
            version_badge: "v1.0",
            time_html: "",
            status_class: "status-running",
            status_label: "RUNNING",
            resources_html: "",
            lan_ip_port_html: "",
            mapped_drives_html: "",
            ports_html: "",
            rollback_html: "",
            start_btn: "",
            stop_btn: "",
            edit_btn: "",
            logs_btn: "",
            autostart_html: "",
        }
    }

    #[test]
    fn test_build_row_html_contains_name_and_status() {
        let html = build_row_html(&make_data());
        assert!(html.contains("my-service"));
        assert!(html.contains("RUNNING"));
        assert!(html.contains("status-running"));
        assert!(html.contains("data-name=\"my-service\""));
    }

    #[test]
    fn test_build_row_html_escapes_html_in_name() {
        let mut data = make_data();
        data.name = "<script>alert(1)</script>";
        let html = build_row_html(&data);
        assert!(
            !html.contains("<script>alert(1)</script>"),
            "raw script tag must not appear unescaped"
        );
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_build_row_html_escapes_js_context_in_remove_handler() {
        let mut data = make_data();
        data.name = "svc'; alert(1); //";
        let html = build_row_html(&data);
        // The apostrophe must be HTML-escaped so the onclick='...'
        // string can't be broken out of by closing the quote early.
        // The escape chain runs js_escape first (replacing `'` with `\'`)
        // and then html_escape, which turns `'` into `&#x27;`.
        assert!(
            html.contains("&#x27;"),
            "apostrophe must be html-escaped to &#x27; so the onclick handler stays well-formed"
        );
        // The injected `alert(1)` is now safely inside the string literal.
        assert!(html.contains("alert(1)"));
    }
}
