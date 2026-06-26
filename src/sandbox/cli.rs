use crate::sandbox::{build_bwrap_command, SandboxConfig};

/// Helper method to parse CLI sandbox arguments manually without large clap crate.
pub fn parse_sandbox_args(args: &[String]) -> Result<String, String> {
    let mut name = String::new();
    let mut appdata = String::new();
    let mut media = None;
    let mut puid = 99;
    let mut pgid = 100;
    let mut gpu = false;
    let mut cmd = String::new();
    let mut extra_binds = Vec::new();
    let mut port = None;
    let mut bind_address = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                if i + 1 >= args.len() { return Err("Missing value for --name".to_string()); }
                name = args[i+1].clone();
                i += 2;
            }
            "--appdata" => {
                if i + 1 >= args.len() { return Err("Missing value for --appdata".to_string()); }
                appdata = args[i+1].clone();
                i += 2;
            }
            "--media" => {
                if i + 1 >= args.len() { return Err("Missing value for --media".to_string()); }
                let val = args[i+1].clone();
                media = if val.trim().is_empty() || val == "-" { None } else { Some(val) };
                i += 2;
            }
            "--puid" => {
                if i + 1 >= args.len() { return Err("Missing value for --puid".to_string()); }
                puid = args[i+1].parse::<u32>().map_err(|_| "Invalid PUID")?;
                i += 2;
            }
            "--pgid" => {
                if i + 1 >= args.len() { return Err("Missing value for --pgid".to_string()); }
                pgid = args[i+1].parse::<u32>().map_err(|_| "Invalid PGID")?;
                i += 2;
            }
            "--gpu" => {
                gpu = true;
                i += 1;
            }
            "--cmd" => {
                if i + 1 >= args.len() { return Err("Missing value for --cmd".to_string()); }
                cmd = args[i+1].clone();
                i += 2;
            }
            "--extra-binds" => {
                if i + 1 >= args.len() { return Err("Missing value for --extra-binds".to_string()); }
                extra_binds = parse_binds_string(&args[i+1])?;
                i += 2;
            }
            "--port" => {
                if i + 1 >= args.len() { return Err("Missing value for --port".to_string()); }
                port = Some(args[i+1].clone());
                i += 2;
            }
            "--bind-address" => {
                if i + 1 >= args.len() { return Err("Missing value for --bind-address".to_string()); }
                bind_address = Some(args[i+1].clone());
                i += 2;
            }
            _ => return Err(format!("Unknown sandbox flag: {}", args[i])),
        }
    }

    build_bwrap_command(&SandboxConfig {
        name,
        appdata_path: appdata,
        media_path: media,
        puid,
        pgid,
        enable_gpu: gpu,
        inner_command: cmd,
        extra_binds,
        port,
        bind_address,
    })
}

pub fn parse_binds_string(s: &str) -> Result<Vec<(String, String)>, String> {
    let mut binds = Vec::new();
    if s.trim().is_empty() {
        return Ok(binds);
    }
    for part in s.split(',') {
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() != 2 {
            return Err(format!("Invalid extra bind format: '{}'. Expected 'host:sandbox'.", part));
        }
        binds.push((subparts[0].to_string(), subparts[1].to_string()));
    }
    Ok(binds)
}
