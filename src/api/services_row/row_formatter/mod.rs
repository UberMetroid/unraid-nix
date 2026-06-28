use crate::process::ServiceStatus;
use crate::config::ProcessComposeConfig;
use crate::api::utils::{HostAddr, get_service_web_port, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};
use crate::api::utils::{html_escape, js_escape};
use crate::unraid::METADATA_DIR;

use super::cells::{
    render_lan_ip_port_cell,
    render_resources_cell, render_autostart_cell,
};

pub mod ports;
pub mod templates;

pub use ports::get_service_ports;

/// Truncates `s` to a maximum of `max_chars` characters, keeping the trailing
/// `keep_chars` characters if truncation is needed. Returns the string with a
/// leading "..." prefix when truncated. Uses `char_indices` to walk UTF-8
/// character boundaries safely — a naive byte slice would panic on multi-byte
/// sequences that span the cutoff.
fn truncate_path_ellipsis(s: &str, max_chars: usize, keep_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let skip = s.chars().count().saturating_sub(keep_chars);
    let mut buf = String::with_capacity(keep_chars + 3);
    buf.push_str("...");
    for (i, c) in s.chars().enumerate() {
        if i >= skip {
            buf.push(c);
        }
    }
    buf
}

pub fn render_service_row(
    s: &ServiceStatus,
    config: &Option<ProcessComposeConfig>,
    host_ips: &[HostAddr],
) -> String {
    let is_running = s.status.to_lowercase() == "running";
    let status_lower = s.status.to_lowercase();
    let is_stopped = status_lower == "stopped"
        || status_lower == "completed"
        || status_lower == "terminating";

    let status_class = if is_running {
        "status-running"
    } else if is_stopped && s.exit_code.unwrap_or(0) == 0 {
        "status-stopped"
    } else {
        "status-failed"
    };

    let status_label = if is_running {
        "RUNNING"
    } else if is_stopped && s.exit_code.unwrap_or(0) == 0 {
        "STOPPED"
    } else {
        "FAILED"
    };

    let cmd = config
        .as_ref()
        .and_then(|c| c.processes.get(&s.name))
        .map(|p| p.command.as_str())
        .unwrap_or("");

    let uri = extract_package_uri(cmd).unwrap_or_else(|| format!("nixpkgs#{}", s.name));
    let version = get_cached_version(&uri);

    let version_badge = if version != "unknown" {
        if let Some(link_url) = get_package_link_url(&uri) {
            format!(
                r#"v<a href="{}" target="_blank" style="color: var(--nix-accent); text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 8px;"></i></a>"#,
                html_escape(&link_url), html_escape(&version)
            )
        } else {
            format!("v{}", html_escape(&version))
        }
    } else {
        "v0.0.0".to_string()
    };

    let port_num = get_service_web_port(&s.name);
    let metadata_file = if crate::store::is_valid_service_name(&s.name) {
        format!("{METADATA_DIR}/{}.json", s.name)
    } else {
        String::new()
    };
    let mut bind_address_override = None;
    let mut appdata_path = None;
    let mut extra_binds_vec = Vec::new();
    let mut gpus_override = None;
    let mut legacy_gpu = None;

    if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            appdata_path = meta.get("appdata")
                .and_then(|v| v.as_str())
                .map(String::from);

            bind_address_override = meta.get("bind_address")
                .and_then(|v| v.as_str())
                .map(String::from);

            gpus_override = meta.get("gpus")
                .and_then(|v| v.as_str())
                .map(String::from);

            legacy_gpu = meta.get("gpu")
                .and_then(|v| v.as_str())
                .map(String::from);

            if let Some(binds_val) = meta.get("extra_binds") {
                if let Some(binds_str) = binds_val.as_str() {
                    if let Ok(parsed_binds) = serde_json::from_str::<serde_json::Value>(binds_str) {
                          if let Some(arr) = parsed_binds.as_array() {
                              for item in arr {
                                  if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|h| h.as_str()), item.get("sandbox").and_then(|s| s.as_str())) {
                                      extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                                  }
                              }
                          }
                      }
                  } else if let Some(arr) = binds_val.as_array() {
                      for item in arr {
                          if let (Some(host), Some(sandbox)) = (item.get("host").and_then(|h| h.as_str()), item.get("sandbox").and_then(|s| s.as_str())) {
                              extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                          }
                      }
                  }
              }
          }
      }

      let lan_ip_port_html = render_lan_ip_port_cell(port_num, is_running, host_ips, &bind_address_override);

      let autostart_enabled = config
          .as_ref()
          .and_then(|c| c.processes.get(&s.name))
          .and_then(|p| p.availability.as_ref())
          .map(|a| a.restart.to_lowercase() == "always")
          .unwrap_or(true);

      let autostart_html = render_autostart_cell(&s.name, autostart_enabled);

      let start_btn = if !is_running {
          format!(r#"<button type="button" class="nix-btn nix-btn-sm" onclick="serviceAction('{}', 'start')" title="Start"><i class="fa fa-play" style="color: #2ecc71;"></i></button>"#, html_escape(&js_escape(&s.name)))
      } else {
          r#"<button type="button" class="nix-btn nix-btn-sm" disabled title="Service is running"><i class="fa fa-play" style="color: var(--nix-text-muted);"></i></button>"#.to_string()
      };

      let stop_btn = if is_running {
          format!(r#"<button type="button" class="nix-btn nix-btn-sm" onclick="serviceAction('{}', 'stop')" title="Stop"><i class="fa fa-stop" style="color: #e74c3c;"></i></button>"#, html_escape(&js_escape(&s.name)))
      } else {
          r#"<button type="button" class="nix-btn nix-btn-sm" disabled title="Service is stopped"><i class="fa fa-stop" style="color: var(--nix-text-muted);"></i></button>"#.to_string()
      };

      let edit_btn = format!(
          r#"<button type="button" class="nix-btn nix-btn-sm" onclick="editService('{}')" title="Edit Config"><i class="fa fa-edit"></i></button>"#,
          html_escape(&js_escape(&s.name))
      );

      let logs_btn = format!(r#"<button type="button" class="nix-btn nix-btn-sm" onclick="openLogs('{}')" title="Logs"><i class="fa fa-file-text-o"></i></button>"#, html_escape(&js_escape(&s.name)));

      let mut mapped_drives = Vec::new();
      if let Some(ref path) = appdata_path {
          if !path.trim().is_empty() {
              mapped_drives.push((path.clone(), "/config".to_string()));
          }
      }
      for (host, sandbox) in extra_binds_vec {
          mapped_drives.push((host, sandbox));
      }

      let mapped_drives_html = if mapped_drives.is_empty() {
          r#"<span style="color: var(--nix-text-muted);">None</span>"#.to_string()
      } else {
          let mut lines = Vec::new();
          for (h, s) in &mapped_drives {
              let h_short = truncate_path_ellipsis(h, 40, 37);
              let s_short = truncate_path_ellipsis(s, 30, 27);
              lines.push(format!(
                  r#"<div style="font-family: monospace; font-size: 10px; color: var(--nix-text-primary); text-overflow: ellipsis; white-space: nowrap; overflow: hidden;" title="{} → {}">{} → {}</div>"#,
                  html_escape(h), html_escape(s), html_escape(&h_short), html_escape(&s_short)
              ));
          }
          lines.join("")
      };

      let ports_list = get_service_ports(&s.name);
      let ports_html = if ports_list.is_empty() {
          r#"<span style="color: var(--nix-text-muted);">None</span>"#.to_string()
      } else {
          let mut lines = Vec::new();
          for p in &ports_list {
              lines.push(format!(
                  r#"<div style="font-family: monospace; font-size: 10px; color: var(--nix-text-primary);">{} → {} (TCP)</div>"#,
                  p.host, p.container
              ));
          }
          lines.join("")
      };

      let rollback_html = r#"<div style="display: flex; align-items: center; justify-content: space-between; width: 100%; height: 16px;">
              <span style="color: var(--nix-text-primary); font-family: monospace; font-size: 10px;">Gen 1 (Active)</span>
              <button type="button" class="nix-btn" style="padding: 1px 4px; font-size: 8px; line-height: 1; min-width: unset; margin: 0; height: 16px; border: 1px solid var(--nix-border-primary); background: var(--nix-bg-tertiary); color: var(--nix-text-secondary); border-radius: 3px; cursor: pointer;" onclick="alert('Rollback support is coming in a future update!')">Rollback</button>
          </div>"#.to_string();

      let time_html = if is_running {
          format!(
              r#"<span style="font-size: 10px; color: var(--nix-text-secondary); font-family: monospace; line-height: 1;">{}</span>"#,
              html_escape(&s.uptime())
          )
      } else {
          String::new()
      };

      let resources_html = render_resources_cell(&s.name, is_running, s.cpu, s.memory, &gpus_override, &legacy_gpu, &s.gpu_stats);

      use super::static_config::get_service_fa_config;
      let cfg = get_service_fa_config(&s.name);

      templates::build_row_html(&templates::RowTemplateData {
          name: &s.name,
          bg: cfg.bg,
          border: cfg.border,
          color: cfg.color,
          icon: &cfg.icon,
          version_badge: &version_badge,
          time_html: &time_html,
          status_class,
          status_label,
          resources_html: &resources_html,
          lan_ip_port_html: &lan_ip_port_html,
          mapped_drives_html: &mapped_drives_html,
          ports_html: &ports_html,
          rollback_html: &rollback_html,
          start_btn: &start_btn,
          stop_btn: &stop_btn,
          edit_btn: &edit_btn,
          logs_btn: &logs_btn,
          autostart_html: &autostart_html,
      })
  }

#[cfg(test)]
mod tests {
    use super::truncate_path_ellipsis;

    #[test]
    fn test_truncate_path_ellipsis_under_limit() {
        let s = "/mnt/cache/appdata/test";
        assert_eq!(truncate_path_ellipsis(s, 40, 37), "/mnt/cache/appdata/test");
    }

    #[test]
    fn test_truncate_path_ellipsis_over_limit() {
        let s = "/mnt/cache/appdata/this-is-a-very-long-path-that-exceeds-forty-chars";
        let result = truncate_path_ellipsis(s, 40, 37);
        assert!(result.starts_with("..."));
        assert_eq!(result.chars().count(), 40);
    }

    #[test]
    fn test_truncate_path_ellipsis_at_exact_boundary() {
        // Exactly 40 chars: should NOT be truncated.
        let s = "a".repeat(40);
        assert_eq!(truncate_path_ellipsis(&s, 40, 37), s);
    }

    #[test]
    fn test_truncate_path_ellipsis_handles_multibyte_utf8() {
        // Multi-byte chars: each is 1 char but multiple bytes. A naive byte
        // slice at the cutoff would panic on a boundary. Counting chars
        // avoids that.
        let s = "/mnt/数据/测试/路径/with-very-long-prefix-so-truncation-kicks-in";
        let result = truncate_path_ellipsis(s, 20, 17);
        assert!(result.starts_with("..."));
        // Result is valid UTF-8 (Rust strings guarantee this, but the test
        // also asserts we got at most 20 chars).
        assert!(result.chars().count() <= 20);
    }
}
