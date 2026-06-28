use crate::config::ProcessComposeConfig;
use crate::api::utils::{get_service_web_port, extract_package_uri};
use crate::api::package::{get_cached_version, get_package_link_url};
use crate::api::utils::html_escape;
use crate::unraid::METADATA_DIR;

/// Truncates `s` to a maximum of `max_chars` characters, keeping the trailing
/// `keep_chars` characters if truncation is needed. Returns the string with a
/// leading "..." prefix when truncated. Uses `char_indices` to walk UTF-8
/// character boundaries safely — a naive byte slice would panic on multi-byte
/// sequences that span the cutoff.
pub(crate) fn truncate_path_ellipsis(s: &str, max_chars: usize, keep_chars: usize) -> String {
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

/// Metadata fields extracted from the service's JSON metadata file.
pub(crate) struct ExtractedMetadata {
    pub(crate) appdata_path: Option<String>,
    pub(crate) bind_address_override: Option<String>,
    pub(crate) gpus_override: Option<String>,
    pub(crate) legacy_gpu: Option<String>,
    pub(crate) extra_binds_vec: Vec<(String, String)>,
}

pub(crate) fn extract_metadata(name: &str) -> ExtractedMetadata {
    if !crate::store::is_valid_service_name(name) {
        return ExtractedMetadata {
            appdata_path: None,
            bind_address_override: None,
            gpus_override: None,
            legacy_gpu: None,
            extra_binds_vec: Vec::new(),
        };
    }
    let metadata_file = format!("{METADATA_DIR}/{name}.json");
    let mut appdata_path = None;
    let mut bind_address_override = None;
    let mut gpus_override = None;
    let mut legacy_gpu = None;
    let mut extra_binds_vec = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
            appdata_path = meta.get("appdata").and_then(|v| v.as_str()).map(String::from);
            bind_address_override = meta.get("bind_address").and_then(|v| v.as_str()).map(String::from);
            gpus_override = meta.get("gpus").and_then(|v| v.as_str()).map(String::from);
            legacy_gpu = meta.get("gpu").and_then(|v| v.as_str()).map(String::from);

            if let Some(binds_val) = meta.get("extra_binds") {
                if let Some(binds_str) = binds_val.as_str() {
                    if let Ok(parsed_binds) = serde_json::from_str::<serde_json::Value>(binds_str) {
                        if let Some(arr) = parsed_binds.as_array() {
                            for item in arr {
                                if let (Some(host), Some(sandbox)) = (
                                    item.get("host").and_then(|h| h.as_str()),
                                    item.get("sandbox").and_then(|s| s.as_str()),
                                ) {
                                    extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                                }
                            }
                        }
                    }
                } else if let Some(arr) = binds_val.as_array() {
                    for item in arr {
                        if let (Some(host), Some(sandbox)) = (
                            item.get("host").and_then(|h| h.as_str()),
                            item.get("sandbox").and_then(|s| s.as_str()),
                        ) {
                            extra_binds_vec.push((host.to_string(), sandbox.to_string()));
                        }
                    }
                }
            }
        }
    }

    ExtractedMetadata {
        appdata_path,
        bind_address_override,
        gpus_override,
        legacy_gpu,
        extra_binds_vec,
    }
}

pub(crate) fn version_badge_for(uri: &str) -> String {
    let version = get_cached_version(uri);
    if version != "unknown" {
        if let Some(link_url) = get_package_link_url(uri) {
            format!(
                r#"v<a href="{}" target="_blank" style="color: var(--nix-accent); text-decoration: none;">{} <i class="fa fa-external-link" style="font-size: 8px;"></i></a>"#,
                html_escape(&link_url), html_escape(&version)
            )
        } else {
            format!("v{}", html_escape(&version))
        }
    } else {
        "v0.0.0".to_string()
    }
}

pub(crate) fn status_fields(status: &str) -> (&'static str, &'static str) {
    let lower = status.to_lowercase();
    let is_running = lower == "running";
    let is_stopped = lower == "stopped" || lower == "completed" || lower == "terminating";

    if is_running {
        ("status-running", "RUNNING")
    } else if is_stopped {
        ("status-stopped", "STOPPED")
    } else {
        ("status-failed", "FAILED")
    }
}

pub(crate) fn derive_uri(cmd: &str, fallback_name: &str) -> String {
    extract_package_uri(cmd).unwrap_or_else(|| format!("nixpkgs#{fallback_name}"))
}

pub(crate) fn get_service_cmd(
    config: &Option<ProcessComposeConfig>,
    name: &str,
) -> Option<String> {
    config
        .as_ref()
        .and_then(|c| c.processes.get(name))
        .map(|p| p.command.clone())
}

pub(crate) fn autodetect_service_port(name: &str) -> Option<u16> {
    get_service_web_port(name)
}

pub(crate) fn autostart_enabled_for(
    config: &Option<ProcessComposeConfig>,
    name: &str,
) -> bool {
    config
        .as_ref()
        .and_then(|c| c.processes.get(name))
        .and_then(|p| p.availability.as_ref())
        .map(|a| a.restart.to_lowercase() == "always")
        .unwrap_or(true)
}

pub(crate) fn collect_mapped_drives(meta: &ExtractedMetadata) -> Vec<(String, String)> {
    let mut mapped_drives = Vec::new();
    if let Some(ref path) = meta.appdata_path {
        if !path.trim().is_empty() {
            mapped_drives.push((path.clone(), "/config".to_string()));
        }
    }
    for (host, sandbox_path) in &meta.extra_binds_vec {
        mapped_drives.push((host.clone(), sandbox_path.clone()));
    }
    mapped_drives
}