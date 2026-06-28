//! Convert the generic `Yaml` value tree into the typed
//! `ProcessComposeConfig` / `ProcessDefinition` structs.

use super::value::{
    expect_bool, expect_map, expect_string, expect_string_seq, expect_u32, expect_u64, Yaml,
};
use crate::config::{
    Availability, LogConfiguration, ProcessComposeConfig, ProcessDefinition, Rotation,
};
use std::collections::HashMap;

pub(super) fn parse_log_configuration(y: &Yaml) -> Result<LogConfiguration, String> {
    let entries = expect_map(y.clone(), "log_configuration")?;
    let mut lc = LogConfiguration {
        add_timestamp: None,
        fields_order: None,
        rotation: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "add_timestamp" => lc.add_timestamp = Some(expect_bool(&v, "add_timestamp")?),
            "fields_order" => lc.fields_order = Some(expect_string_seq(&v, "fields_order")?),
            "rotation" => lc.rotation = Some(parse_rotation(&v)?),
            other => return Err(format!("unknown log_configuration field: {}", other)),
        }
    }
    Ok(lc)
}

pub(super) fn parse_rotation(y: &Yaml) -> Result<Rotation, String> {
    let entries = expect_map(y.clone(), "rotation")?;
    let mut r = Rotation {
        max_size_mb: None,
        max_backups: None,
        compress: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "max_size_mb" => r.max_size_mb = Some(expect_u64(&v, "max_size_mb")?),
            "max_backups" => r.max_backups = Some(expect_u32(&v, "max_backups")?),
            "compress" => r.compress = Some(expect_bool(&v, "compress")?),
            other => return Err(format!("unknown rotation field: {}", other)),
        }
    }
    Ok(r)
}

pub(super) fn parse_processes_map(y: &Yaml) -> Result<HashMap<String, ProcessDefinition>, String> {
    let entries = expect_map(y.clone(), "processes")?;
    let mut out = HashMap::new();
    for (name, v) in entries {
        out.insert(name, parse_process(&v)?);
    }
    Ok(out)
}

pub(super) fn parse_process(y: &Yaml) -> Result<ProcessDefinition, String> {
    let entries = expect_map(y.clone(), "process")?;
    let mut p = ProcessDefinition {
        command: String::new(),
        availability: None,
        environment: None,
        log_location: None,
        log_configuration: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "command" => p.command = expect_string(&v, "command")?,
            "availability" => p.availability = Some(parse_availability(&v)?),
            "environment" => p.environment = Some(expect_string_seq(&v, "process.environment")?),
            "log_location" => p.log_location = Some(expect_string(&v, "log_location")?),
            "log_configuration" => p.log_configuration = Some(parse_log_configuration(&v)?),
            other => return Err(format!("unknown process field: {}", other)),
        }
    }
    if p.command.is_empty() {
        return Err("process is missing required 'command'".to_string());
    }
    Ok(p)
}

pub(super) fn parse_availability(y: &Yaml) -> Result<Availability, String> {
    let entries = expect_map(y.clone(), "availability")?;
    let mut a = Availability {
        restart: String::new(),
        backoff_seconds: None,
        max_restarts: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "restart" => a.restart = expect_string(&v, "restart")?,
            "backoff_seconds" => a.backoff_seconds = Some(expect_u64(&v, "backoff_seconds")?),
            "max_restarts" => a.max_restarts = Some(expect_u32(&v, "max_restarts")?),
            other => return Err(format!("unknown availability field: {}", other)),
        }
    }
    if a.restart.is_empty() {
        return Err("availability is missing required 'restart'".to_string());
    }
    Ok(a)
}

pub(super) fn to_config(root: Yaml) -> Result<ProcessComposeConfig, String> {
    let map = expect_map(root, "root")?;
    let version = map
        .iter()
        .find(|(k, _)| k == "version")
        .map(|(_, v)| expect_string(v, "version"))
        .transpose()?
        .unwrap_or_else(|| "0.5".to_string());
    let environment = map
        .iter()
        .find(|(k, _)| k == "environment")
        .map(|(_, v)| expect_string_seq(v, "environment"))
        .transpose()?;
    let log_configuration = map
        .iter()
        .find(|(k, _)| k == "log_configuration")
        .map(|(_, v)| parse_log_configuration(v))
        .transpose()?;
    let processes = map
        .iter()
        .find(|(k, _)| k == "processes")
        .ok_or_else(|| "missing required 'processes' section".to_string())
        .and_then(|(_, v)| parse_processes_map(v))?;

    Ok(ProcessComposeConfig {
        version,
        environment,
        log_configuration,
        processes,
    })
}
