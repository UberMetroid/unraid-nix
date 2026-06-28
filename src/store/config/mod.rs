use chrono::Local;
use std::io::Write;
use crate::unraid::NIX_CFG_PATH;

const BYTES_PER_GB: u64 = 1 << 30;

pub fn log_event(level: &str, msg: &str) {
    log_event_to_path("/var/log/nix-plugin.log", level, msg, 10 * 1024 * 1024);
}

fn log_event_to_path(log_path: &str, level: &str, msg: &str, max_size: u64) {

    struct LockGuard {
        path: String,
        active: bool,
    }
    impl Drop for LockGuard {
        fn drop(&mut self) {
            if self.active {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }

    let lock_path = format!("{log_path}.lock");
    let mut guard = LockGuard { path: lock_path.clone(), active: false };

    let mut delay = std::time::Duration::from_millis(5);
    for _ in 0..5 {
        if std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .is_ok()
        {
            guard.active = true;
            break;
        }
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, std::time::Duration::from_millis(80));
    }

    if !guard.active && std::env::var_os("NIX_DEBUG").is_some() {
        eprintln!("[NIX_DEBUG] Failed to acquire log lock for {log_path}");
    }

    if guard.active {
        if let Ok(metadata) = std::fs::metadata(log_path) {
            if metadata.len() > max_size {
                let p2 = format!("{log_path}.2");
                let p3 = format!("{log_path}.3");
                if std::path::Path::new(&p2).exists() {
                    let _ = std::fs::rename(&p2, &p3);
                }
                let p1 = format!("{log_path}.1");
                if std::path::Path::new(&p1).exists() {
                    let _ = std::fs::rename(&p1, &p2);
                }
                let _ = std::fs::rename(log_path, &p1);
            }
        }
    }

    let safe_level = sanitize_log_token(level);
    let safe_msg = sanitize_log_token(msg);

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("{now} [{safe_level}] {safe_msg}\n");

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path) {
            let _ = file.write_all(line.as_bytes());
        }

    if log_path == "/var/log/nix-plugin.log" {
        let _ = std::process::Command::new("logger")
            .args(["-t", "nix-plugin", &format!("[{safe_level}] {safe_msg}")])
            .stdin(std::process::Stdio::null())
            .output();
    }

    #[cfg(not(test))]
    {
        if safe_level == "ERROR" || safe_level == "WARN" || std::env::var_os("NIX_DEBUG").is_some() {
            eprintln!("[{safe_level}] {safe_msg}");
        }
    }
}

/// Sanitize a log token (level or message) so a malicious caller cannot
/// forge extra log lines by embedding CR/LF or bracket markers.
fn sanitize_log_token(s: &str) -> String {
    s.replace(['\n', '\r'], " ")
        .replace('[', "(")
        .replace(']', ")")
}

/// Validation check for the persistent store path.
pub fn validate_store_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Nix store path cannot be empty.".to_string());
    }
    if path.starts_with("/boot") {
        return Err("Nix store path cannot be located on the boot flash drive (/boot).".to_string());
    }
    Ok(())
}

pub fn read_cfg_val(key: &str, default: &str) -> String {
    let map = crate::unraid::parse_ini_file(NIX_CFG_PATH);
    map.get(key).cloned().unwrap_or_else(|| default.to_string())
}

pub fn read_allow_source_builds() -> bool {
    read_cfg_val("ALLOW_SOURCE_BUILDS", "no") == "yes"
}

pub fn generate_nix_conf_content(
    allow_source: bool,
    build_cores: &str,
    build_jobs: &str,
    gc_min_free_gb: u64,
    gc_max_free_gb: u64,
) -> Result<String, String> {
    let (jobs_val, cores_val) = if allow_source {
        let j = if build_jobs == "0" {
            let total = std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4);
            let half = total.saturating_add(1) / 2;
            std::cmp::max(1, half).to_string()
        } else {
            build_jobs.to_string()
        };
        (j, build_cores.to_string())
    } else {
        ("0".to_string(), "0".to_string())
    };

    let min_free_bytes = gc_min_free_gb.checked_mul(BYTES_PER_GB)
        .ok_or_else(|| "min_free_gb overflow".to_string())?;
    let max_free_bytes = gc_max_free_gb.checked_mul(BYTES_PER_GB)
        .ok_or_else(|| "max_free_gb overflow".to_string())?;

    Ok(format!(
        "experimental-features = nix-command flakes\nmax-jobs = {jobs_val}\ncores = {cores_val}\nmin-free = {min_free_bytes}\nmax-free = {max_free_bytes}\n"
    ))
}

pub fn is_valid_service_name(name: &str) -> bool {
    if name.is_empty() { return false; }
    if name.starts_with('.') || name.ends_with('.') || name.contains("..") { return false; }
    name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

#[cfg(test)]
mod tests;
