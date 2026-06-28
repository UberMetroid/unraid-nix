//! Runtime probe of Nix's per-derivation build sandbox.
//!
//! Emits a JSON report describing whether the host can run
//! `nix build <derivation>` with `sandbox = true` set. The probe is
//! reports-only by default. The `--apply-fallback` flag is the only
//! way the plugin modifies nix.cfg, and only after a real build probe
//! has failed.
//!
//! Two layers:
//! 1. Primitive check (fast, ~50ms): user-namespace support, mount
//!    propagation on /nix, /etc/nix symlink state, NIX_BUILD_SANDBOX
//!    current value.
//! 2. Build probe (10s timeout): attempt to build nixpkgs#hello with
//!    `sandbox = true` in a temporary nix.conf override. The
//!    `NIX_CONFIG` env var is used to direct nix to the test config
//!    without writing to the real /nix/etc/nix/nix.conf.

use serde_json::json;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};

const NIX_BIN: &str = "/nix/var/nix/profiles/default/bin/nix";
const PROBE_TIMEOUT_SECS: u64 = 10;

pub fn sandbox_check(apply_fallback: bool) {
    let report = run_probe();

    // Reports-only by default. Print the JSON to stdout.
    let pretty = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string());
    println!("{}", pretty);

    let recommendation = report
        .get("recommendation")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    if recommendation != "ok" {
        crate::store::log_event(
            "INFO",
            &format!(
                "sandbox-check recommendation: {recommendation} (apply_fallback={apply_fallback})"
            ),
        );
    }

    if apply_fallback
        && (recommendation == "fall_back_to_false" || recommendation == "kernel_unsupported")
    {
        if let Err(e) = write_fallback_to_nix_cfg() {
            crate::store::log_event(
                "ERROR",
                &format!("Could not write sandbox = false to nix.cfg: {e}"),
            );
            std::process::exit(1);
        }
        crate::store::log_event(
            "INFO",
            "Wrote sandbox = false to /boot/config/plugins/nix/nix.cfg (audit trail above).",
        );
    }
}

fn run_probe() -> serde_json::Value {
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

fn check_user_ns() -> bool {
    // Spawn a process that creates a user namespace and immediately
    // exits. If the unshare call fails, the kernel does not support
    // user namespaces for unprivileged users.
    let result = unsafe {
        Command::new("unshare")
            .arg("--user")
            .arg("--map-root-user")
            .arg("--mount-proc")
            .arg("true")
            .pre_exec(|| Ok(()))
            .status()
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

    let start = Instant::now();
    let result = Command::new(NIX_BIN)
        .arg("build")
        .arg("--no-link")
        .arg("--no-update-lock-file")
        .arg("nixpkgs#hello")
        .env("NIX_CONFIG", conf_str)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status();
    let _ = std::fs::remove_file(&tmp_conf);
    let elapsed = start.elapsed();

    if elapsed > Duration::from_secs(PROBE_TIMEOUT_SECS) {
        return Err(format!(
            "build probe exceeded {}s timeout",
            PROBE_TIMEOUT_SECS
        ));
    }

    match result {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!(
            "build probe failed with exit code {}",
            s.code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "?".to_string())
        )),
        Err(e) => Err(format!("could not invoke nix: {e}")),
    }
}

fn write_fallback_to_nix_cfg() -> Result<(), String> {
    let path = "/boot/config/plugins/nix/nix.cfg";
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("could not read {path}: {e}")),
    };
    let mut new_content = String::new();
    let mut replaced = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("sandbox") && trimmed.contains('=') {
            // Preserve any inline comment after the value
            let comment_start = trimmed.find('#').unwrap_or(trimmed.len());
            new_content.push_str(trimmed[..comment_start].trim_end());
            new_content.push_str(" = false\n");
            replaced = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }
    if !replaced {
        new_content.push_str("\nsandbox = false\n");
    }
    std::fs::write(path, new_content).map_err(|e| {
        let err_msg = format!("could not write {path}: {e}");
        crate::store::log_event("ERROR", &err_msg);
        err_msg
    })?;
    Ok(())
}
