use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::process::ServiceStatus;

use super::rows::{get_sorted_statuses, render_single_row};

const STATE_PATH: &str = "/var/run/nix-dashboard-state.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct DashboardState {
    /// Monotonic counter; lets the client short-circuit when nothing changed.
    version: u64,
    /// Per-service fingerprint from the last emission; used to detect which rows actually changed.
    fingerprints: HashMap<String, String>,
    /// Names observed at the last emission; anything here that is not in the current snapshot is reported as `removed`.
    names: Vec<String>,
}

#[derive(Debug, Default, PartialEq)]
struct DiffResult {
    pub version: u64,
    pub changed: Vec<(String, String)>,
    pub removed: Vec<String>,
}

/// Renders a JSON patch describing only the rows whose status, cpu,
/// memory, uptime, gpu_active, or io counters changed since the last
/// successful diff emission. The frontend tracks `version` and replaces
/// only the affected `<tr>` elements; this avoids re-serializing all
/// rows on every 3-second poll.
pub fn dashboard_diff(api_port: u16, since: u64) -> String {
    let old_state = read_state();
    let statuses = match get_sorted_statuses(api_port) {
        Ok(s) => s,
        Err(_) => {
            return serde_json::json!({"version": old_state.version, "changed": [], "removed": []})
                .to_string();
        }
    };

    let result = compute_diff(&old_state, &statuses, since);
    write_state(&DashboardState {
        version: result.version,
        fingerprints: statuses
            .iter()
            .map(|s| (s.name.clone(), fingerprint(s)))
            .collect(),
        names: statuses.iter().map(|s| s.name.clone()).collect(),
    });
    render_diff_payload(&result)
}

/// Pure diff computation; given the previous state and the current
/// statuses, returns the rows whose HTML must be re-rendered and the
/// names that disappeared. If `since < old.version` (client is too
/// far behind or first call with empty state), emits every current row
/// as a full refresh — we have no fingerprints for `since`.
fn compute_diff(old: &DashboardState, current: &[ServiceStatus], since: u64) -> DiffResult {
    let new_version = old.version.saturating_add(1);
    let mut changed: Vec<(String, String)> = Vec::new();

    if since < old.version {
        for s in current {
            changed.push((s.name.clone(), render_single_row(s)));
        }
    } else {
        for s in current {
            let fp = fingerprint(s);
            if old.fingerprints.get(&s.name) != Some(&fp) {
                changed.push((s.name.clone(), render_single_row(s)));
            }
        }
    }

    let current_set: HashSet<&str> = current.iter().map(|s| s.name.as_str()).collect();
    let removed: Vec<String> = old
        .names
        .iter()
        .filter(|n| !current_set.contains(n.as_str()))
        .cloned()
        .collect();
    DiffResult {
        version: new_version,
        changed,
        removed,
    }
}

fn render_diff_payload(result: &DiffResult) -> String {
    let changed_json: Vec<serde_json::Value> = result
        .changed
        .iter()
        .map(|(name, html)| serde_json::json!({ "name": name, "html": html }))
        .collect();
    serde_json::json!({
        "version": result.version,
        "changed": changed_json,
        "removed": result.removed,
    })
    .to_string()
}

fn fingerprint(s: &ServiceStatus) -> String {
    let cpu_q = (s.cpu.unwrap_or(0.0) * 100.0) as i64;
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        s.status,
        cpu_q,
        s.memory.unwrap_or(0),
        s.uptime_nanoseconds.unwrap_or(0),
        s.io_read.unwrap_or(0),
        s.io_write.unwrap_or(0),
        s.gpu_active.unwrap_or(false) as u8,
    )
}

fn read_state() -> DashboardState {
    let Ok(content) = std::fs::read_to_string(STATE_PATH) else {
        return DashboardState::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn write_state(state: &DashboardState) {
    let path = Path::new(STATE_PATH);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(state) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(name: &str, status: &str, cpu: Option<f32>, mem: Option<u64>) -> ServiceStatus {
        ServiceStatus {
            name: name.to_string(),
            status: status.to_string(),
            pid: Some(1),
            cpu,
            memory: mem,
            uptime_nanoseconds: Some(60_000_000_000),
            exit_code: None,
            gpu_active: Some(false),
            io_read: Some(0),
            io_write: Some(0),
            gpu_stats: None,
        }
    }

    #[test]
    fn test_fingerprint_changes_with_status_cpu_memory_and_is_stable_otherwise() {
        let base = status("a", "Running", Some(1.0), Some(100));
        let same = status("a", "Running", Some(1.0), Some(100));
        let stopped = status("a", "Stopped", Some(1.0), Some(100));
        let cpu_diff = status("a", "Running", Some(2.5), Some(100));
        let mem_diff = status("a", "Running", Some(1.0), Some(200));

        let fp = fingerprint(&base);
        assert_eq!(fp, fingerprint(&same), "identical inputs must hash equal");
        assert_ne!(
            fp,
            fingerprint(&stopped),
            "status flip must change fingerprint"
        );
        assert_ne!(
            fp,
            fingerprint(&cpu_diff),
            "cpu change must change fingerprint"
        );
        assert_ne!(
            fp,
            fingerprint(&mem_diff),
            "memory change must change fingerprint"
        );
    }

    #[test]
    fn test_read_state_returns_default_when_file_missing() {
        let s = read_state();
        assert!(s.fingerprints.is_empty());
    }

    #[test]
    fn test_compute_diff_empty_old_state_with_since_zero_returns_all_rows() {
        // First-ever call: every current row must be emitted so the
        // client can populate its cache from scratch.
        let result = compute_diff(
            &DashboardState::default(),
            &[
                status("radarr", "Running", Some(1.0), Some(100)),
                status("sonarr", "Stopped", None, None),
            ],
            0,
        );
        assert_eq!(result.version, 1);
        assert_eq!(result.changed.len(), 2);
        assert!(result.removed.is_empty());
    }

    #[test]
    fn test_compute_diff_no_changes_advances_version_with_empty_changed() {
        // Same statuses twice: empty `changed` (no DOM patches) but
        // version still advances so the client can detect freshness.
        let fp = fingerprint(&status("radarr", "Running", Some(1.0), Some(100)));
        let old = DashboardState {
            version: 1,
            fingerprints: [("radarr".to_string(), fp)].into_iter().collect(),
            names: vec!["radarr".to_string()],
        };
        let result = compute_diff(
            &old,
            &[status("radarr", "Running", Some(1.0), Some(100))],
            1,
        );
        assert_eq!(result.version, 2);
        assert!(result.changed.is_empty());
        assert!(result.removed.is_empty());
    }

    #[test]
    fn test_compute_diff_status_transition_emits_only_changed_row() {
        let fp = fingerprint(&status("radarr", "Running", Some(1.0), Some(100)));
        let old = DashboardState {
            version: 1,
            fingerprints: [("radarr".to_string(), fp)].into_iter().collect(),
            names: vec!["radarr".to_string()],
        };
        let result = compute_diff(
            &old,
            &[status("radarr", "Stopped", Some(1.0), Some(100))],
            1,
        );
        assert_eq!(result.changed.len(), 1);
        assert_eq!(result.changed[0].0, "radarr");
        assert!(result.changed[0].1.contains(">Stopped<"));
    }

    #[test]
    fn test_compute_diff_disappeared_service_reported_in_removed() {
        // Service stopped being managed: its name must appear in
        // `removed` so the frontend can drop its `<tr>` from the DOM.
        let old = DashboardState {
            version: 5,
            fingerprints: [
                ("radarr".to_string(), "fp1".to_string()),
                ("sonarr".to_string(), "fp2".to_string()),
            ]
            .into_iter()
            .collect(),
            names: vec!["radarr".to_string(), "sonarr".to_string()],
        };
        let result = compute_diff(
            &old,
            &[status("radarr", "Running", Some(1.0), Some(100))],
            5,
        );
        assert_eq!(result.removed, vec!["sonarr".to_string()]);
        assert_eq!(result.changed.len(), 1);
    }

    #[test]
    fn test_compute_diff_client_too_far_behind_returns_full_snapshot() {
        // Client lost several polls (or server restarted and lost
        // state): we can't diff against an unknown snapshot, so emit
        // every current row as a full refresh.
        let old = DashboardState {
            version: 10,
            fingerprints: HashMap::new(),
            names: vec![],
        };
        let result = compute_diff(
            &old,
            &[
                status("radarr", "Running", Some(1.0), Some(100)),
                status("sonarr", "Running", Some(1.0), Some(100)),
            ],
            3,
        );
        assert_eq!(result.changed.len(), 2);
        assert!(result.removed.is_empty());
    }

    #[test]
    fn test_render_diff_payload_shape() {
        let result = DiffResult {
            version: 7,
            changed: vec![("radarr".to_string(), "<tr></tr>".to_string())],
            removed: vec!["oldname".to_string()],
        };
        let json: serde_json::Value = serde_json::from_str(&render_diff_payload(&result)).unwrap();
        assert_eq!(json["version"], 7);
        assert_eq!(json["changed"][0]["name"], "radarr");
        assert_eq!(json["changed"][0]["html"], "<tr></tr>");
        assert_eq!(json["removed"][0], "oldname");
    }
}
