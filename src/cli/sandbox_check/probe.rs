//! Probe helpers for the Nix build-sandbox check command.
//!
//! Pure data-acquisition functions. The public `sandbox_check` function
//! in `cli::sandbox_check` handles printing the report and the
//! `--apply-fallback` side effect on nix.cfg.

use serde_json::json;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};
use crate::util::process::{run_with_timeout, run_with_timeout_status};

pub(crate) const NIX_BIN: &str = "/nix/var/nix/profiles/default/bin/nix";
pub(crate) const PROBE_TIMEOUT_SECS: u64 = 10;

pub(crate) fn run_probe() -> serde_json::Value {
    let user_ns_ok = check_user_ns();
    let mount_propagation = check_mount_propagation();
    let nix_conf_sandbox = read_existing_nix_conf_sandbox();
    let build_ok = probe_build_sandbox();
    let build_error = if build_ok.is_ok() {
        None
    } else {
        Some(build_ok.as_ref().err().unwrap().clone())
    };

    let build_ok_bool = build_ok.is_ok();
    let recommendation = if !user_ns_ok {
        "kernel_unsupported"
    } else if !build_ok_bool {
        "fall_back_to_false"
    } else {
        "ok"
    };

    json!({
        "user_ns_supported": user_ns_ok,
        "mount_propagation_on_nix": mount_propagation,
        "existing_nix_conf_sandbox_setting": nix_conf_sandbox,
        "build_probe_ok": build_ok_bool,
        "build_probe_error": build_error,
        "recommendation": recommendation,
    })
}

#[allow(unsafe_code)]
fn check_user_ns() -> bool {
    // Spawn a process that creates a user namespace and immediately
    // exits. If the unshare call fails, the kernel does not support
    // user namespaces for unprivileged users. `.pre_exec` is unsafe by
    // signature; we allow the lint at function scope.
    let result = unsafe {
        let mut cmd = Command::new("unshare");
        cmd.arg("--user")
            .arg("--map-root-user")
            .arg("--mount-proc")
            .arg("true")
            .pre_exec(|| Ok(()));
        run_with_timeout_status(&mut cmd, Duration::from_secs(5))
    };
    match result {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

fn check_mount_propagation() -> String {
    // Read /proc/self/mountinfo for the /nix entry and report its
    // propagation flags. The fields are separated by spaces; the
    // propagation fields are at fixed positions in each record.
    let Ok(content) = std::fs::read_to_string("/proc/self/mountinfo") else {
        return "unknown".to_string();
    };
    for line in content.lines() {
        let mut fields = line.split(' ').filter(|s| !s.is_empty());
        let _mount_id = fields.next();
        let _parent = fields.next();
        let _device = fields.next();
        let _root = fields.next();
        let _mount_point = fields.next();
        let _fs_type = fields.next();
        let _source = fields.next();
        let _super_opts = fields.next();
        if let Some(opts) = fields.next() {
            // mountinfo optional fields start here; "shared:N" / "master:N" /
            // "propagate_from:N" / "unbindable" / "private" all live in this
            // block. Look for the propagation marker.
            if line.contains(" /nix ") || line.starts_with("/nix ") {
                return if opts.contains("shared:") {
                    "shared".to_string()
                } else if opts.contains("master:") {
                    "master".to_string()
                } else {
                    "private".to_string()
                };
            }
        }
    }
    "unknown".to_string()
}

fn read_existing_nix_conf_sandbox() -> String {
    let path = "/nix/etc/nix/nix.conf";
    let Ok(content) = std::fs::read_to_string(path) else {
        return "unknown".to_string();
    };
    for line in content.lines() {
        let trimmed = line.trim();
        // Match `sandbox` then any combination of `=`, whitespace, value.
        // `sandbox = true` => rest = "= true" => after trim + strip = "true".
        // `sandbox=true` => rest = "=true" => after trim + strip = "true".
        // `sandbox =` => rest = "=" => empty after trim => skip.
        if let Some(rest) = trimmed.strip_prefix("sandbox") {
            let value = rest.trim().trim_start_matches('=').trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    "unset".to_string()
}

fn probe_build_sandbox() -> Result<(), String> {
    if !std::path::Path::new(NIX_BIN).exists() {
        return Err(format!("{} not found; install Nix first", NIX_BIN));
    }

    // Use a temporary NIX_CONFIG env var to point nix at a private
    // config that has `sandbox = true` set. This does not touch the
    // real /nix/etc/nix/nix.conf.
    let tmp_conf = std::env::temp_dir().join("nix-sandbox-probe.conf");
    let conf_content = "sandbox = true\n";
    if let Err(e) = std::fs::write(&tmp_conf, conf_content) {
        return Err(format!("could not write temp probe config: {e}"));
    }
    let conf_str = match tmp_conf.to_str() {
        Some(s) => s.to_string(),
        None => {
            return Err("temp path is not valid UTF-8".to_string());
        }
    };

    let result = {
        let mut cmd = Command::new(NIX_BIN);
        cmd.arg("build")
            .arg("--no-link")
            .arg("--no-update-lock-file")
            .arg("nixpkgs#hello")
            .env("NIX_CONFIG", conf_str)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());
        run_with_timeout(&mut cmd, Duration::from_secs(PROBE_TIMEOUT_SECS))
    };
    let _ = std::fs::remove_file(&tmp_conf);

    match result {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(format!(
            "build probe failed with exit code {}",
            out.status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "?".to_string())
        )),
        Err(e) if e.contains("timeout") => Err(format!(
            "build probe exceeded {}s timeout",
            PROBE_TIMEOUT_SECS
        )),
        Err(e) => Err(format!("could not invoke nix: {e}")),
    }
}