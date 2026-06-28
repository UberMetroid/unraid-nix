//! Canonical process-compose YAML serializer.

use crate::config::{LogConfiguration, ProcessComposeConfig};

/// Serialize a `ProcessComposeConfig` to canonical process-compose YAML.
///
/// Deterministic: process map keys are emitted in sorted order so that
/// `serialize -> parse -> serialize` yields the same string.
pub fn serialize_config(config: &ProcessComposeConfig) -> String {
    let mut out = String::new();
    out.push_str(&format!("version: {}\n", yaml_scalar(&config.version)));

    if let Some(env) = &config.environment {
        out.push_str("environment:\n");
        for item in env {
            out.push_str(&format!("- {}\n", yaml_scalar(item)));
        }
    }

    if let Some(lc) = &config.log_configuration {
        out.push_str("log_configuration:\n");
        push_log_configuration(&mut out, lc, 2);
    }

    out.push_str("processes:\n");
    let mut names: Vec<&String> = config.processes.keys().collect();
    names.sort();
    for name in names {
        let proc = &config.processes[name];
        out.push_str(&format!("  {}:\n", name));
        out.push_str(&format!("    command: {}\n", yaml_scalar(&proc.command)));
        if let Some(av) = &proc.availability {
            out.push_str("    availability:\n");
            out.push_str(&format!("      restart: {}\n", yaml_scalar(&av.restart)));
            if let Some(b) = av.backoff_seconds {
                out.push_str(&format!("      backoff_seconds: {}\n", b));
            }
            if let Some(m) = av.max_restarts {
                out.push_str(&format!("      max_restarts: {}\n", m));
            }
        }
        if let Some(env) = &proc.environment {
            out.push_str("    environment:\n");
            for item in env {
                out.push_str(&format!("    - {}\n", yaml_scalar(item)));
            }
        }
        if let Some(loc) = &proc.log_location {
            out.push_str(&format!("    log_location: {}\n", yaml_scalar(loc)));
        }
        if let Some(lc) = &proc.log_configuration {
            out.push_str("    log_configuration:\n");
            push_log_configuration(&mut out, lc, 4);
        }
    }
    out
}

fn push_log_configuration(out: &mut String, lc: &LogConfiguration, indent: usize) {
    let pad = " ".repeat(indent);
    if let Some(t) = lc.add_timestamp {
        out.push_str(&format!("{}add_timestamp: {}\n", pad, t));
    }
    if let Some(fo) = &lc.fields_order {
        out.push_str(&format!("{}fields_order:\n", pad));
        for item in fo {
            out.push_str(&format!("{}- {}\n", pad, yaml_scalar(item)));
        }
    }
    if let Some(r) = &lc.rotation {
        out.push_str(&format!("{}rotation:\n", pad));
        let rp = " ".repeat(indent + 2);
        if let Some(m) = r.max_size_mb {
            out.push_str(&format!("{}max_size_mb: {}\n", rp, m));
        }
        if let Some(b) = r.max_backups {
            out.push_str(&format!("{}max_backups: {}\n", rp, b));
        }
        if let Some(c) = r.compress {
            out.push_str(&format!("{}compress: {}\n", rp, c));
        }
    }
}

/// Render a string scalar in double-quoted YAML form. We always quote so the
/// output round-trips unambiguously into the same string value (unquoted
/// `0.5` would otherwise be unparseable back into a string in some YAML
/// implementations). Only control characters and the double quote itself need
/// to be escaped.
pub(super) fn yaml_scalar(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}