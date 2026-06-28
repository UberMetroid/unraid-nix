use chrono::Local;
use fs2::FileExt;
use std::io::Write;
use std::time::Duration;
use crate::util::process::run_with_timeout;

pub fn log_event(level: &str, msg: &str) {
    log_event_to_path("/var/log/nix-plugin.log", level, msg, 10 * 1024 * 1024);
}

pub(crate) fn log_event_to_path(log_path: &str, level: &str, msg: &str, max_size: u64) {
    let lock_path = format!("{log_path}.lock");

    let lock_file: Option<std::fs::File> = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .ok();

    let lock_held = match lock_file.as_ref() {
        Some(f) => {
            let mut ok = false;
            for attempt in 0..3 {
                ok = f.try_lock_exclusive().is_ok();
                if ok { break; }
                if attempt < 2 {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            if !ok {
                eprintln!("[WARN] Could not acquire log lock for {log_path} after retries; proceeding without lock");
            }
            ok
        }
        None => false,
    };

    if lock_held {
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
        let _ = {
            let mut cmd = std::process::Command::new("logger");
            cmd.args(["-t", "nix-plugin", &format!("[{safe_level}] {safe_msg}")])
                .stdin(std::process::Stdio::null());
            run_with_timeout(&mut cmd, Duration::from_secs(2))
        };
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